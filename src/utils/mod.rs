use std::time::SystemTime;

pub fn unixepoch() -> i64 {
    SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64
}
