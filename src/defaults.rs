use std::time::Duration;

/// Default time between Events when an EventList is sent
pub fn default_time_between() -> u64 {
    300
}

pub fn halt_events_if_error() -> bool {
    false
}

pub fn client_update_interval() -> Duration {
    Duration::from_secs(25)
}
