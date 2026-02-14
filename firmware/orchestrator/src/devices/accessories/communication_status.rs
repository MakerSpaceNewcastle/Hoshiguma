use crate::logic::safety::monitor::{NEW_MONITOR_STATUS, ObservedSeverity};
use defmt::{Format, unwrap, warn};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::Publisher};
use embassy_time::{Duration, Instant, Timer};
use hoshiguma_core::types::{MonitorKind, Severity};

enum CommunicationStatus {
    Ok { last: Instant },
    Failed { since: Instant, times: usize },
}

pub(super) struct CommunicationStatusReporter {
    status: CommunicationStatus,
    severity: ObservedSeverity,
    monitor: MonitorKind,
    warn_timeout: Duration,
    critical_timeout: Duration,
    monitor_tx: Publisher<'static, CriticalSectionRawMutex, (MonitorKind, Severity), 8, 1, 10>,
}

impl CommunicationStatusReporter {
    pub(super) fn new(
        monitor: MonitorKind,
        warn_timeout: Duration,
        critical_timeout: Duration,
    ) -> Self {
        let monitor_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

        Self {
            status: CommunicationStatus::Failed {
                since: Instant::now(),
                times: 0,
            },
            severity: ObservedSeverity::default(),
            monitor,
            warn_timeout,
            critical_timeout,
            monitor_tx,
        }
    }

    pub(super) async fn comm_good(&mut self) {
        self.status = CommunicationStatus::Ok {
            last: Instant::now(),
        };

        self.evaluate().await;
    }

    pub(super) async fn comm_fail(&mut self) -> CommunicationFailureAction {
        const ATTEMPTS_BEFORE_GIVE_UP: usize = 10;

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

        if attempts > ATTEMPTS_BEFORE_GIVE_UP {
            warn!("Giving up after {} communication attempts", attempts);
            self.evaluate().await;
            CommunicationFailureAction::GiveUp
        } else {
            // Wait before retry
            Timer::after_millis(10).await;

            CommunicationFailureAction::Retry
        }
    }

    pub(super) async fn evaluate(&mut self) {
        let severity = match self.status {
            CommunicationStatus::Ok { last } => {
                if Instant::now().saturating_duration_since(last) > self.warn_timeout {
                    self.status = CommunicationStatus::Failed {
                        since: Instant::now(),
                        times: 1,
                    };
                    Severity::Warning
                } else {
                    Severity::Normal
                }
            }
            CommunicationStatus::Failed { since, times: _ } => {
                if Instant::now().saturating_duration_since(since) > self.critical_timeout {
                    Severity::Critical
                } else {
                    Severity::Warning
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

#[derive(Format, PartialEq, Eq)]
pub(super) enum CommunicationFailureAction {
    Retry,
    GiveUp,
}
