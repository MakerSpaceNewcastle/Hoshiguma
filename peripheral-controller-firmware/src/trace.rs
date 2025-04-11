use core::{cell::RefCell, fmt::Write, future::poll_fn, task::Poll};
use defmt::{debug, info};
use embassy_executor::raw::task_from_waker;
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_time::{Duration, Instant, Ticker};
use heapless::{FnvIndexMap, String};

#[derive(Debug, defmt::Format)]
enum TaskState {
    Idle,
    Running,
    Waiting,
}

#[derive(Debug, defmt::Format)]
struct TaskInfo {
    name: Option<&'static str>,
    executor_id: u32,
    total_idle: Duration,
    total_running: Duration,
    total_waiting: Duration,
    total_wakes: u64,
    current_state_since: Instant,
    current_state: TaskState,
}

impl TaskInfo {
    fn new(executor_id: u32) -> Self {
        Self {
            name: None,
            executor_id,
            total_idle: Duration::default(),
            total_running: Duration::default(),
            total_waiting: Duration::default(),
            total_wakes: 0,
            current_state_since: Instant::now(),
            current_state: TaskState::Idle,
        }
    }

    fn name(&self) -> &'static str {
        self.name.unwrap_or("unknown")
    }

    fn reset_counters(&mut self, now: Instant) {
        self.total_idle = Duration::default();
        self.total_running = Duration::default();
        self.total_waiting = Duration::default();
        self.total_wakes = 0;
        self.current_state_since = now;
    }

    fn update(&mut self, now: Instant, new_state: TaskState) {
        let duration = now.saturating_duration_since(self.current_state_since);

        match self.current_state {
            TaskState::Idle => {
                self.total_idle += duration;
            }
            TaskState::Running => {
                self.total_running += duration;
            }
            TaskState::Waiting => {
                self.total_waiting += duration;
            }
        }

        self.current_state_since = now;
        self.current_state = new_state;
    }
}

static TASKS: CriticalSectionMutex<RefCell<FnvIndexMap<u32, TaskInfo, 32>>> =
    CriticalSectionMutex::new(RefCell::new(FnvIndexMap::new()));
static EXECUTOR_NAMES: CriticalSectionMutex<RefCell<FnvIndexMap<u32, &'static str, 8>>> =
    CriticalSectionMutex::new(RefCell::new(FnvIndexMap::new()));

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_new(executor_id: u32, task_id: u32) {
    info!("New task {} executor {}", task_id, executor_id);

    TASKS.lock(|tasks| {
        tasks
            .borrow_mut()
            .insert(task_id, TaskInfo::new(executor_id))
            .unwrap();
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_ready_begin(_executor_id: u32, task_id: u32) {
    TASKS.lock(|tasks| {
        if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
            task.update(Instant::now(), TaskState::Waiting);
            task.total_wakes = task.total_wakes.saturating_add(1);
        }
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_exec_begin(_executor_id: u32, task_id: u32) {
    TASKS.lock(|tasks| {
        if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
            task.update(Instant::now(), TaskState::Running);
        }
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_exec_end(_executor_id: u32, task_id: u32) {
    TASKS.lock(|tasks| {
        if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
            task.update(Instant::now(), TaskState::Idle);
        }
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_poll_start(_executor_id: u32) {}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_executor_idle(_executor_id: u32) {}

pub(crate) fn name_executor(executor_id: u32, name: &'static str) {
    EXECUTOR_NAMES.lock(|executor_names| {
        executor_names
            .borrow_mut()
            .insert(executor_id, name)
            .unwrap();
    });
}

pub(crate) async fn name_task(name: &'static str) {
    let task_id: u32 = poll_fn(|cx| Poll::Ready(task_from_waker(cx.waker()).as_id())).await;

    TASKS.lock(|tasks| {
        if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
            task.name = Some(name);
        }
    });
}

#[embassy_executor::task]
pub(crate) async fn task() {
    name_task("task rprt").await;

    let mut ticker = Ticker::every(Duration::from_secs(15));

    let mut last_report_time = Instant::now();
    let mut table_line_buffer = String::<120>::new();

    loop {
        ticker.next().await;

        info!("│Task        │Executor│State│Idle  │Run   │Wait  │#Wakes│");
        info!("├────────────┼────────┼─────┼──────┼──────┼──────┼──────┤");
        TASKS.lock(|tasks| {
            let now = Instant::now();
            let ticks = (now.saturating_duration_since(last_report_time)).as_ticks();

            for task in tasks.borrow_mut().values_mut() {
                let executor_name = EXECUTOR_NAMES
                    .lock(|executor_names| {
                        executor_names.borrow_mut().get(&task.executor_id).copied()
                    })
                    .unwrap_or("unknown");

                table_line_buffer
                    .write_fmt(format_args!(
                        "│{:<12}│{:<8}│{}│{:.4}│{:.4}│{:.4}│{:>6}│",
                        task.name(),
                        executor_name,
                        match task.current_state {
                            TaskState::Idle => "Idle ",
                            TaskState::Running => "Run  ",
                            TaskState::Waiting => "Wait ",
                        },
                        task.total_idle.as_ticks() as f32 / ticks as f32,
                        task.total_running.as_ticks() as f32 / ticks as f32,
                        task.total_waiting.as_ticks() as f32 / ticks as f32,
                        task.total_wakes
                    ))
                    .unwrap();

                task.reset_counters(now);

                info!("{}", table_line_buffer);
                table_line_buffer.clear();
            }

            debug!("Ticks in interval: {}", ticks);
            last_report_time = now;
        });
    }
}
