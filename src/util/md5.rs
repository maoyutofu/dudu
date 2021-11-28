/// 对传入的字符串参数进行 MD5 加密后返回
pub fn hash_password(password: String) -> String {
    let digest = md5::compute(password.as_bytes());
    format!("{:x}", digest)
}
