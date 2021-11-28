use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ipc {
    pub id: i32,
    pub key: String,
    pub name: String,
    pub rtsp: String,
    pub rtmp: String,
    pub enable: i32, // 0 禁用推流  1 启用推流
    pub reason: Option<String>,
    pub retry_count: i32,
    pub create_time: i64,
    pub update_time: Option<i64>,
}

impl Ipc {
    pub fn new(
        id: i32,
        key: String,
        name: String,
        rtsp: String,
        rtmp: String,
        enable: i32,
        reason: Option<String>,
        retry_count: i32,
        create_time: i64,
        update_time: Option<i64>,
    ) -> Self {
        Ipc {
            id,
            key,
            name,
            rtsp,
            rtmp,
            enable,
            reason,
            retry_count,
            create_time,
            update_time,
        }
    }
}

use crate::db;
use rusqlite::{params, Result};

const CREATE_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS tb_ipc (id INTEGER NOT NULL,key VARCHAR(32) NOT NULL UNIQUE,name VARCHAR(50) NOT NULL,rtsp VARCHAR(255) NOT NULL,rtmp VARCHAR(255) NOT NULL,enable TINYINT NOT NULL DEFAULT 0,reason VARCHAR(255) NULL,retry_count INTEGER NOT NULL DEFAULT 0,create_time BIGINT NOT NULL,update_time BIGINT NULL,PRIMARY KEY (id))";
const INSERT_SQL: &str =
    "INSERT INTO tb_ipc(key, name, rtsp, rtmp, enable, create_time) VALUES(?,?,?,?,?,?)";
const UPDATE_SQL: &str =
    "UPDATE tb_ipc SET key=?, name=?, rtsp=?, rtmp=?, enable=?, reason=?, retry_count=?, update_time=? WHERE id=?";
const DELETE_SQL: &str = "DELETE FROM tb_ipc WHERE id=?";
const GET_BY_ID_SQL: &str = "SELECT * FROM tb_ipc WHERE id=?";
const GET_BY_KEY_SQL: &str = "SELECT * FROM tb_ipc WHERE key=?";
const GET_LIST_SQL: &str = "SELECT * FROM tb_ipc WHERE 1=1";
const COUNT_SQL: &str = "SELECT COUNT(1) FROM tb_ipc WHERE 1=1";

const MAX_ROWS: i32 = 64;

#[derive(Clone)]
pub struct IpcService;

impl IpcService {
    pub fn new() -> Result<Self> {
        db::create_table(CREATE_TABLE_SQL)?;
        Ok(IpcService {})
    }

    /// 执行Insert SQL往数据库中添加一条Ipc数据
    pub fn insert(&self, ipc: Ipc) -> Result<usize> {
        db::conn()?.execute(
            INSERT_SQL,
            params![
                ipc.key,
                ipc.name,
                ipc.rtsp,
                ipc.rtmp,
                ipc.enable,
                ipc.create_time
            ],
        )
    }

    /// 执行Update SQL修改数据库中的Ipc数据
    pub fn update(&self, ipc: Ipc) -> Result<usize> {
        db::conn()?.execute(
            UPDATE_SQL,
            params![
                ipc.key,
                ipc.name,
                ipc.rtsp,
                ipc.rtmp,
                ipc.enable,
                ipc.reason,
                ipc.retry_count,
                ipc.update_time,
                ipc.id
            ],
        )
    }

    /// 执行Delete SQL从数据库中删除一条Ipc数据
    pub fn delete(&self, id: i32) -> Result<usize> {
        db::conn()?.execute(DELETE_SQL, params![id])
    }

    /// 通过id来获取一条Ipc数据
    pub fn get(&self, id: i32) -> Result<Option<Ipc>> {
        let conn = db::conn()?;
        let mut stmt = conn.prepare(GET_BY_ID_SQL)?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Ipc::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })?;
        let row = match rows.next() {
            None => None,
            Some(row) => Some(row?),
        };
        Ok(row)
    }

    /// 通过key来获取一条Ipc数据
    pub fn get_by_key(&self, key: String) -> Result<Option<Ipc>> {
        let conn = db::conn()?;
        let mut stmt = conn.prepare(GET_BY_KEY_SQL)?;
        let mut rows = stmt.query_map(params![key], |row| {
            Ok(Ipc::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })?;
        let row = match rows.next() {
            None => None,
            Some(row) => Some(row?),
        };
        Ok(row)
    }

    /// 获取Ipc列表
    pub fn get_list(&self, page: u32, rows: u32, keyword: Option<String>) -> Result<Vec<Ipc>> {
        let conn = db::conn()?;
        let mut sql = String::from(GET_LIST_SQL);
        if keyword.is_some() {
            let keyword = keyword.unwrap();
            sql += &format!(
                " AND key LIKE '%{0}%' OR name LIKE '%{0}%' OR rtsp LIKE '%{0}%' OR rtmp LIKE '%{0}%'",
                keyword
            );
        }
        sql += " LIMIT ? OFFSET ?";
        let mut stmp = conn.prepare(&sql)?;
        let offset = (page - 1) * rows;
        let rows = stmp.query_map(params![rows, offset], |row| {
            Ok(Ipc::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })?;
        let mut row_list: Vec<Ipc> = Vec::new();
        for row in rows {
            row_list.push(row?);
        }
        Ok(row_list)
    }

    /// 获取启用状态Ipc列表
    pub fn get_enable_list(&self) -> Result<Vec<Ipc>> {
        let conn = db::conn()?;
        let mut sql = String::from(GET_LIST_SQL);
        sql += " AND enable = 1";
        sql += " LIMIT ? OFFSET ?";
        let mut stmp = conn.prepare(&sql)?;
        let rows = MAX_ROWS;
        let offset = (1 - 1) * rows;
        let rows = stmp.query_map(params![rows, offset], |row| {
            Ok(Ipc::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })?;
        let mut row_list: Vec<Ipc> = Vec::new();
        for row in rows {
            row_list.push(row?);
        }
        Ok(row_list)
    }

    /// 获取异常状态Ipc列表
    pub fn get_list_by_reason(&self, less_retry_count: u32) -> Result<Vec<Ipc>> {
        let conn = db::conn()?;
        let mut sql = String::from(GET_LIST_SQL);
        sql += " AND enable = 0 AND reason IS NOT NULL AND retry_count < ?";
        sql += " LIMIT ? OFFSET ?";
        let mut stmp = conn.prepare(&sql)?;
        let rows = MAX_ROWS;
        let offset = (1 - 1) * rows;
        let rows = stmp.query_map(params![less_retry_count, rows, offset], |row| {
            Ok(Ipc::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })?;
        let mut row_list: Vec<Ipc> = Vec::new();
        for row in rows {
            row_list.push(row?);
        }
        Ok(row_list)
    }

    /// 统计IPC数量
    pub fn count(&self) -> Result<u64> {
        let conn = db::conn()?;
        let mut stmp = conn.prepare(COUNT_SQL)?;
        let count: i64 = stmp.query_row(params![], |row| {
            let count: i64 = row.get(0)?;
            Ok(count)
        })?;
        Ok(count as u64)
    }

    /// 获取启用状态Ipc的数量
    pub fn count_enable(&self) -> Result<u64> {
        let conn = db::conn()?;
        let mut sql = String::from(COUNT_SQL);
        sql += " AND enable = 1";
        let mut stmp = conn.prepare(&sql)?;
        let count: i64 = stmp.query_row(params![], |row| {
            let count: i64 = row.get(0)?;
            Ok(count)
        })?;
        Ok(count as u64)
    }

    /// 获取异常状态Ipc的数量
    pub fn count_reason(&self) -> Result<u64> {
        let conn = db::conn()?;
        let mut sql = String::from(COUNT_SQL);
        sql += " AND enable = 0 AND reason IS NOT NULL";
        let mut stmp = conn.prepare(&sql)?;
        let count: i64 = stmp.query_row(params![], |row| {
            let count: i64 = row.get(0)?;
            Ok(count)
        })?;
        Ok(count as u64)
    }
}
