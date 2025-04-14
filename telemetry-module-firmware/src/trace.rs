use core::{cell::RefCell, fmt::Write, future::poll_fn, task::Poll};
use defmt::{info, Format};
use embassy_executor::raw::task_from_waker;
use embassy_sync::blocking_mutex::CriticalSectionMutex;
use embassy_time::{Duration, Instant, Ticker};
use heapless::{FnvIndexMap, String};

#[derive(Clone, Debug, Format)]
enum TaskState {
    Idle,
    Running,
    Waiting,
}

#[derive(Clone, Debug, Format)]
struct TaskInfo {
    name: Option<&'static str>,
    total_idle: Duration,
    total_running: Duration,
    total_waiting: Duration,
    total_wakes: u64,
    current_state_since: Instant,
    current_state: TaskState,
}

impl Default for TaskInfo {
    fn default() -> Self {
        Self {
            name: None,
            total_idle: Duration::default(),
            total_running: Duration::default(),
            total_waiting: Duration::default(),
            total_wakes: 0,
            current_state_since: Instant::now(),
            current_state: TaskState::Idle,
        }
    }
}

impl TaskInfo {
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

static TASKS_CORE_0: CriticalSectionMutex<RefCell<FnvIndexMap<u32, TaskInfo, 32>>> =
    CriticalSectionMutex::new(RefCell::new(FnvIndexMap::new()));

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_new(executor_id: u32, task_id: u32) {
    info!("New task {} executor {}", task_id, executor_id);

    TASKS_CORE_0.lock(|tasks| {
        tasks
            .borrow_mut()
            .insert(task_id, TaskInfo::default())
            .unwrap();
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_end(_executor_id: u32, task_id: u32) {
    info!("Task {} ended", task_id);

    TASKS_CORE_0.lock(|tasks| {
        tasks.borrow_mut().remove(&task_id).unwrap();
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_ready_begin(_executor_id: u32, task_id: u32) {
    TASKS_CORE_0.lock(|tasks| {
        if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
            task.update(Instant::now(), TaskState::Waiting);
            task.total_wakes = task.total_wakes.saturating_add(1);
        }
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_exec_begin(_executor_id: u32, task_id: u32) {
    TASKS_CORE_0.lock(|tasks| {
        if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
            task.update(Instant::now(), TaskState::Running);
        }
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_task_exec_end(_executor_id: u32, task_id: u32) {
    TASKS_CORE_0.lock(|tasks| {
        if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
            task.update(Instant::now(), TaskState::Idle);
        }
    });
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_poll_start(_executor_id: u32) {}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn _embassy_trace_executor_idle(_executor_id: u32) {}

pub(crate) async fn name_task(name: &'static str) {
    assert!(name.len() <= 20, "Task name too long");

    let task_id: u32 = poll_fn(|cx| Poll::Ready(task_from_waker(cx.waker()).as_id())).await;

    TASKS_CORE_0.lock(|tasks| {
        if let Some(task) = tasks.borrow_mut().get_mut(&task_id) {
            task.name = Some(name);
        }
    });
}

#[embassy_executor::task]
pub(crate) async fn task() {
    name_task("task report").await;

    let mut ticker = Ticker::every(Duration::from_secs(15));

    let mut table_line_buffer = String::<120>::new();

    let mut last_report_time_core_0 = Instant::now();

    loop {
        ticker.next().await;

        let (core_0_tasks, core_0_ticks) = TASKS_CORE_0.lock(|tasks| {
            let now = Instant::now();
            let ticks = (now.saturating_duration_since(last_report_time_core_0)).as_ticks();
            let sample = tasks.borrow().clone();
            for task in tasks.borrow_mut().values_mut() {
                task.reset_counters(now);
            }
            last_report_time_core_0 = now;
            (sample, ticks)
        });

        info!("│Task                │State│Idle  │Run   │Wait  │#Wakes│");
        info!("├────────────────────┼─────┼──────┼──────┼──────┼──────┤");

        for task in core_0_tasks.values() {
            print_task_line(task, core_0_ticks, &mut table_line_buffer);
        }
    }
}

fn print_task_line(task: &TaskInfo, core_ticks: u64, buffer: &mut String<120>) {
    buffer
        .write_fmt(format_args!(
            "│{:<20}│{}│{:.4}│{:.4}│{:.4}│{:>6}│",
            task.name(),
            match task.current_state {
                TaskState::Idle => "Idle ",
                TaskState::Running => "Run  ",
                TaskState::Waiting => "Wait ",
            },
            task.total_idle.as_ticks() as f32 / core_ticks as f32,
            task.total_running.as_ticks() as f32 / core_ticks as f32,
            task.total_waiting.as_ticks() as f32 / core_ticks as f32,
            task.total_wakes
        ))
        .unwrap();

    info!("{}", buffer);
    buffer.clear();
}
