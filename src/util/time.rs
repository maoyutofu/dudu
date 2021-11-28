use std::time;

/// 获取当前系统的时间戳并返回
pub fn current_timestamp() -> u128 {
    match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Err(e) => panic!("{}", e),
        Ok(d) => d.as_millis(),
    }
}
