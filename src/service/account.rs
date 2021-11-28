use crate::util;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub uid: i32,
    pub username: String,
    pub password: String,
    pub token: String,
    pub create_time: i64,
    pub update_time: Option<i64>,
}

impl Account {
    pub fn new(
        uid: i32,
        username: String,
        password: String,
        token: String,
        create_time: i64,
        update_time: Option<i64>,
    ) -> Self {
        Account {
            uid,
            username,
            password,
            token,
            create_time,
            update_time,
        }
    }
}

use crate::db;
use rusqlite::{params, Result};

const CREATE_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS tb_account (uid INTEGER NOT NULL,username VARCHAR(15) NOT NULL UNIQUE,password VARCHAR(32) NOT NULL,token VARCHAR(32) NOT NULL UNIQUE,create_time BIGINT NOT NULL,update_time BIGINT NULL,PRIMARY KEY (uid))";
const HAVE_DATA_SQL: &str = "SELECT 1 FROM tb_account LIMIT 1";
const INSERT_SQL: &str =
    "INSERT INTO tb_account(username, password, token, create_time) VALUES(?,?,?,?)";
const UPDATE_SQL: &str =
    "UPDATE tb_account SET username=?, password=?, token=?, update_time=? WHERE uid=?";
const CHANGE_PASSWORD_SQL: &str = "UPDATE tb_account SET password=?, update_time=? WHERE uid=?";
const GET_BY_ID_SQL: &str = "SELECT * FROM tb_account WHERE uid=?";
const GET_BY_USERNAME_SQL: &str = "SELECT * FROM tb_account WHERE username=?";
const GET_BY_TOKEN_SQL: &str = "SELECT * FROM tb_account WHERE token=?";

#[derive(Clone)]
pub struct AccountService {}

impl AccountService {
    pub fn new() -> Result<Self> {
        db::create_table(CREATE_TABLE_SQL)?;
        Ok(AccountService {})
    }

    /// 数据库初始化方法，包括初始化默认登录的用户信息
    pub fn init_data(&self) -> Result<usize, rusqlite::Error> {
        let conn = db::conn()?;
        let mut stmt = conn.prepare(HAVE_DATA_SQL)?;
        match stmt.exists(params![]) {
            Err(e) => Err(e),
            Ok(exists) => {
                if !exists {
                    let account = Account::new(
                        0,
                        "admin".to_string(),
                        "e10adc3949ba59abbe56e057f20f883e".to_string(),
                        util::uuid::token(),
                        util::time::current_timestamp() as i64,
                        None,
                    );
                    match self.insert(account) {
                        Err(e) => Err(e),
                        Ok(_) => Ok(1),
                    }
                } else {
                    Ok(0)
                }
            }
        }
    }

    /// 执行 Insert SQL添加一条Account数据到数据库中
    pub fn insert(&self, account: Account) -> Result<usize> {
        db::conn()?.execute(
            INSERT_SQL,
            params![
                account.username,
                account.password,
                account.token,
                account.create_time
            ],
        )
    }

    /// 执行Update SQL修改数据库中的Account数据
    pub fn update(&self, account: Account) -> Result<usize> {
        db::conn()?.execute(
            UPDATE_SQL,
            params![
                account.username,
                account.password,
                account.token,
                account.update_time,
                account.uid
            ],
        )
    }

    /// 执行Update SQL修改数据库中的Account的密码
    pub fn change_password(&self, password: String, uid: i32) -> Result<usize> {
        db::conn()?.execute(
            CHANGE_PASSWORD_SQL,
            params![password, util::time::current_timestamp() as i64, uid],
        )
    }

    /// 根据uid来获取一个Account信息
    pub fn get(&self, uid: i32) -> Result<Option<Account>> {
        let conn = db::conn()?;
        let mut stmt = conn.prepare(GET_BY_ID_SQL)?;
        let mut rows = stmt.query_map(params![uid], |row| {
            Ok(Account::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;
        let row = match rows.next() {
            None => None,
            Some(row) => Some(row?),
        };
        Ok(row)
    }

    /// 根据username来获取一个Account信息
    pub fn get_by_username(&self, username: String) -> Result<Option<Account>> {
        let conn = db::conn()?;
        let mut stmt = conn.prepare(GET_BY_USERNAME_SQL)?;
        let mut rows = stmt.query_map(params![username], |row| {
            Ok(Account::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;
        let row = match rows.next() {
            None => None,
            Some(row) => Some(row?),
        };
        Ok(row)
    }

    /// 根据token来获取一个Account信息
    pub fn get_by_token(&self, token: String) -> Result<Option<Account>> {
        let conn = db::conn()?;
        let mut stmt = conn.prepare(GET_BY_TOKEN_SQL)?;
        let mut rows = stmt.query_map(params![token], |row| {
            Ok(Account::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;
        let row = match rows.next() {
            None => None,
            Some(row) => Some(row?),
        };
        Ok(row)
    }
}
