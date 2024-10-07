use std::time::{SystemTime, UNIX_EPOCH};

/// ID type used in general
pub type IdType = u64;

/// We store time as unix time
pub type TimeType = u64;

/// As 64-bit integer for sqlite
pub fn now() -> TimeType {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("error calculating duration since unix epoch")
        .as_millis() as TimeType
}
