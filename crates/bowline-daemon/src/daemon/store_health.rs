use std::cell::Cell;
use std::fmt;

pub(super) struct StoreHealth {
    consecutive_failures: Cell<u32>,
    recovery_pending_after_status_publish: Cell<bool>,
}

impl StoreHealth {
    pub(super) fn new() -> Self {
        Self {
            consecutive_failures: Cell::new(0),
            recovery_pending_after_status_publish: Cell::new(false),
        }
    }

    pub(super) fn record<T, E: fmt::Display>(
        &self,
        context: &'static str,
        result: Result<T, E>,
    ) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.consecutive_failures
                    .set(self.consecutive_failures.get().saturating_add(1));
                self.recovery_pending_after_status_publish.set(false);
                eprintln!("bowline-daemon store write failed ({context}): {error}");
                None
            }
        }
    }

    pub(super) fn mark_degraded_status_written(&self) {
        if self.is_degraded() {
            self.recovery_pending_after_status_publish.set(true);
        }
    }

    pub(super) fn recover_after_status_publish(&self) {
        if self.recovery_pending_after_status_publish.replace(false) {
            self.consecutive_failures.set(0);
        }
    }

    pub(super) fn is_degraded(&self) -> bool {
        self.consecutive_failures.get() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_health_degrades_after_write_failure_and_recovers_after_status_publish() {
        let health = StoreHealth::new();

        let failed: Option<()> = health.record("test", Err("locked"));
        assert_eq!(failed, None);
        assert!(health.is_degraded());

        assert_eq!(health.record::<_, &str>("test", Ok(7)), Some(7));
        assert!(health.is_degraded());

        health.mark_degraded_status_written();
        assert!(health.is_degraded());

        health.recover_after_status_publish();
        assert!(!health.is_degraded());
    }
}
