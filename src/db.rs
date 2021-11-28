use rusqlite::{params, Connection, Error, Result};

/// 执行数据库连接操作
pub fn conn() -> Result<Connection, Error> {
    Ok(Connection::open("dudu.db")?)
}

/// 根据传入的建表sql语句创建表
pub fn create_table(sql: &str) -> Result<usize> {
    conn()?.execute(&sql, params![])
}
