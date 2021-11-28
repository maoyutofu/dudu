use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::result::Result;
use crate::service;
use crate::util;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginInfoReq {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginInfoResp {
    pub token: String,
}

#[post("/api/login")]
pub async fn login(
    service: web::Data<Arc<service::Service>>,
    session: Session,
    login_info_req: web::Json<LoginInfoReq>,
) -> impl Responder {
    let username = login_info_req.username.to_string();
    let password = login_info_req.password.to_string();
    let result = match service.account_service.get_by_username(username) {
        Err(e) => serde_json::to_string(&Result::error_description(
            Result::DB_OPERATION_ERROR,
            &e.to_string(),
        )),
        Ok(account) => match account.clone() {
            None => serde_json::to_string(&Result::error(Result::USER_PASSWORD_ERROR)),
            Some(db_account) => {
                let password = util::md5::hash_password(password);
                if password == db_account.password {
                    match session.set("uid", db_account.uid) {
                        Err(e) => serde_json::to_string(&Result::error_description(
                            Result::SESSION_SET_ERROR,
                            &e.to_string(),
                        )),
                        Ok(_) => {
                            let resp = LoginInfoResp {
                                token: db_account.token,
                            };
                            serde_json::to_string(&Result::success_return_data(resp))
                        }
                    }
                } else {
                    serde_json::to_string(&Result::error(Result::USER_PASSWORD_ERROR))
                }
            }
        },
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(result.unwrap())
}
