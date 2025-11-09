use crate::logic::safety::monitor::{ObservedSeverity, NEW_MONITOR_STATUS};
use defmt::{unwrap, warn};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::Publisher};
use embassy_time::{Duration, Instant};
use hoshiguma_protocol::{peripheral_controller::types::MonitorKind, types::Severity};

enum CommunicationStatus {
    Ok { last: Instant },
    Failed { since: Instant, times: usize },
}

pub(super) struct CommunicationStatusReporter {
    status: CommunicationStatus,
    severity: ObservedSeverity,
    monitor: MonitorKind,
    monitor_tx: Publisher<'static, CriticalSectionRawMutex, (MonitorKind, Severity), 8, 1, 9>,
}

impl CommunicationStatusReporter {
    pub(super) fn new(monitor: MonitorKind) -> Self {
        let monitor_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

        Self {
            status: CommunicationStatus::Failed {
                since: Instant::now(),
                times: 0,
            },
            severity: ObservedSeverity::default(),
            monitor,
            monitor_tx,
        }
    }

    pub(super) async fn comm_good(&mut self) {
        self.status = CommunicationStatus::Ok {
            last: Instant::now(),
        };

        self.evaluate().await;
    }

    pub(super) async fn comm_fail(&mut self) -> bool {
        self.status = match self.status {
            CommunicationStatus::Ok { last: _ } => CommunicationStatus::Failed {
                since: Instant::now(),
                times: 1,
            },
            CommunicationStatus::Failed { since, times } => CommunicationStatus::Failed {
                since,
                times: times.saturating_add(1),
            },
        };

        let attempts = match self.status {
            CommunicationStatus::Ok { last: _ } => 0,
            CommunicationStatus::Failed { since: _, times } => times,
        };

        if attempts > 3 {
            warn!("Giving up after {} communication attempts", attempts);
            self.evaluate().await;
            true
        } else {
            false
        }
    }

    pub(super) async fn evaluate(&mut self) {
        const WARN_TIMEOUT: Duration = Duration::from_secs(3);
        const CRITICAL_TIMEOUT: Duration = Duration::from_secs(10);

        let severity = match self.status {
            CommunicationStatus::Ok { last } => {
                if Instant::now().saturating_duration_since(last) > WARN_TIMEOUT {
                    self.status = CommunicationStatus::Failed {
                        since: Instant::now(),
                        times: 1,
                    };
                    Severity::Warn
                } else {
                    Severity::Normal
                }
            }
            CommunicationStatus::Failed { since, times: _ } => {
                if Instant::now().saturating_duration_since(since) > CRITICAL_TIMEOUT {
                    Severity::Critical
                } else {
                    Severity::Warn
                }
            }
        };

        self.severity
            .update_and_async(severity, |severity| async {
                self.monitor_tx
                    .publish((self.monitor.clone(), severity))
                    .await;
            })
            .await;
    }
}
