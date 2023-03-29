use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_timestamp_8_hours_from_now() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    // expires after 8 hours
    let eighthoursfromnow = since_the_epoch + Duration::from_secs(8 * 3600);
    eighthoursfromnow.as_secs()
}
