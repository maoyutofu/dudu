use uuid::Uuid;

/// 使用 UUID 产生一个新的 token 并返回
pub fn token() -> String {
    let mut buf = [b'!'; 36];
    Uuid::new_v4()
        .to_simple()
        .encode_lower(&mut buf)
        .to_string()
}
