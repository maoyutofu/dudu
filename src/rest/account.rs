use actix_web::http::HeaderValue;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::result::Result;
use crate::service;
use crate::util;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordInfoReq {
    pub old_password: String,
    pub new_password: String,
}

#[post("/api/change-password")]
pub async fn change_password(
    service: web::Data<Arc<service::Service>>,
    change_password_req: web::Json<ChangePasswordInfoReq>,
    req: web::HttpRequest,
) -> impl Responder {
    let value = HeaderValue::from_str("").unwrap();
    let token = req.headers().get("token").unwrap_or(&value);
    let old_password = change_password_req.old_password.to_string();
    let new_password = change_password_req.new_password.to_string();

    let result = match service
        .account_service
        .get_by_token(token.to_str().unwrap().to_string())
    {
        Err(e) => serde_json::to_string(&Result::error_description(
            Result::DB_OPERATION_ERROR,
            &e.to_string(),
        )),
        Ok(account) => {
            let db_account = account.unwrap();
            if db_account.password == util::md5::hash_password(old_password) {
                match service
                    .account_service
                    .change_password(util::md5::hash_password(new_password), db_account.uid)
                {
                    Err(e) => serde_json::to_string(&Result::error_description(
                        Result::DB_OPERATION_ERROR,
                        &e.to_string(),
                    )),
                    Ok(_) => serde_json::to_string(&Result::success()),
                }
            } else {
                serde_json::to_string(&Result::error(Result::OLD_PASSWORD_ERROR))
            }
        }
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(result.unwrap())
}
