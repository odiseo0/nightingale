use std::time::SystemTime;

// Domain rules receive time through this port instead of reading the system clock
// directly (Master Plan §11.3).
pub trait Clock {
    fn now(&self) -> SystemTime;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedClock(SystemTime);

    impl Clock for FixedClock {
        fn now(&self) -> SystemTime {
            self.0
        }
    }

    #[test]
    fn port_returns_injected_time() {
        let instant = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000);
        let clock = FixedClock(instant);

        assert_eq!(clock.now(), instant);
    }

    #[test]
    fn system_clock_reports_real_time() {
        let before = SystemTime::now();

        assert!(SystemClock.now() >= before);
    }
}
