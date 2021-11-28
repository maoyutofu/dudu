use serde::Serialize;
use std::boxed::Box;

#[derive(Serialize)]
pub struct Result<T> {
    code: i32,
    msg: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

pub struct Error {
    code: i32,
    message: &'static str,
}

impl Result<()> {
    // 0 操作成功
    pub const SUCCESS: Error = Error {
        code: 0,
        message: "Success",
    };

    // 10000 http 业务错误相关
    pub const INVALID_PARAMETER: Error = Error {
        code: 10001,
        message: "Invalid parameter",
    };

    pub const DATA_NOT_FOUND: Error = Error {
        code: 10002,
        message: "Data not found",
    };

    pub const ALREADY_PUSHING: Error = Error {
        code: 10003,
        message: "Already pushing",
    };

    pub const NOT_PUSHING: Error = Error {
        code: 10004,
        message: "Not pushing",
    };

    pub const USER_PASSWORD_ERROR: Error = Error {
        code: 10005,
        message: "Wrong user name or password",
    };

    pub const OLD_PASSWORD_ERROR: Error = Error {
        code: 10006,
        message: "Old password error",
    };

    // 50000 程序错误相关
    pub const SESSION_SET_ERROR: Error = Error {
        code: 50001,
        message: "Session set error",
    };

    // 60000 数据库错误相关
    pub const DB_OPERATION_ERROR: Error = Error {
        code: 60001,
        message: "Database operational error",
    };

    pub fn success() -> Self {
        let e = Result::SUCCESS;
        Result {
            code: e.code,
            msg: e.message,
            data: None,
        }
    }

    pub fn error(e: Error) -> Self {
        Result {
            code: e.code,
            msg: e.message,
            data: None,
        }
    }

    pub fn error_description(e: Error, msg: &str) -> Self {
        let msg = format!("{} {}", e.message, msg);
        Result {
            code: e.code,
            msg: Box::leak(msg.into_boxed_str()),
            data: None,
        }
    }
}

impl<T> Result<T> {
    pub fn success_return_data(data: T) -> Self {
        let e = Result::SUCCESS;
        Result {
            code: e.code,
            msg: e.message,
            data: Some(data),
        }
    }
}

#[derive(Serialize)]
pub struct Page<T> {
    total: u64,
    rows: Option<T>,
}

impl<T> Page<T> {
    pub fn new(total: u64, rows: T) -> Self {
        Page {
            total: total,
            rows: Some(rows),
        }
    }
}
