use actix::prelude::*;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use serde::{Deserialize, Serialize};

use async_std::task;

use crate::my_actor;
use crate::result::Page;
use crate::result::Result;
use crate::service;
use crate::service::ipc;
use crate::util;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct IpcInfoReq {
    pub id: Option<i32>,
    pub key: String,
    pub name: String,
    pub rtsp: String,
    pub rtmp: String,
}

#[derive(Serialize, Deserialize)]
pub struct PagingInfoReq {
    pub page: Option<u32>,
    pub rows: Option<u32>,
    pub keyword: Option<String>,
}

#[post("/api/ipc")]
pub async fn add_ipc(
    service: web::Data<Arc<service::Service>>,
    ipc_info_req: web::Json<IpcInfoReq>,
) -> impl Responder {
    let create_time = util::time::current_timestamp();

    let ipc = ipc::Ipc::new(
        0,
        ipc_info_req.key.to_string(),
        ipc_info_req.name.to_string(),
        ipc_info_req.rtsp.to_string(),
        ipc_info_req.rtmp.to_string(),
        0,
        None,
        0,
        create_time as i64,
        None,
    );
    let result = match service.ipc_service.insert(ipc) {
        Ok(_) => Result::success(),
        Err(e) => Result::error_description(Result::DB_OPERATION_ERROR, &e.to_string()),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap())
}

#[put("/api/ipc")]
pub async fn update_ipc(
    service: web::Data<Arc<service::Service>>,
    ipc_info_req: web::Json<IpcInfoReq>,
) -> impl Responder {
    let result = match ipc_info_req.id {
        None => Result::error_description(Result::INVALID_PARAMETER, "id"),
        Some(id) => match service.ipc_service.get(id) {
            Err(e) => Result::error_description(Result::DB_OPERATION_ERROR, &e.to_string()),
            Ok(ipc) => match ipc.clone() {
                None => Result::error(Result::DATA_NOT_FOUND),
                Some(mut db_ipc) => {
                    if db_ipc.enable == 1 {
                        Result::error(Result::ALREADY_PUSHING)
                    } else {
                        let update_time = util::time::current_timestamp();
                        db_ipc.key = ipc_info_req.key.to_string();
                        db_ipc.name = ipc_info_req.name.to_string();
                        db_ipc.rtsp = ipc_info_req.rtsp.to_string();
                        db_ipc.rtmp = ipc_info_req.rtmp.to_string();
                        db_ipc.update_time = Some(update_time as i64);
                        match service.ipc_service.update(db_ipc) {
                            Ok(_) => Result::success(),
                            Err(e) => Result::error_description(
                                Result::DB_OPERATION_ERROR,
                                &e.to_string(),
                            ),
                        }
                    }
                }
            },
        },
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap())
}

#[get("/api/ipc/{id}/start")]
pub async fn ipc_publish_start(
    service: web::Data<Arc<service::Service>>,
    addr: web::Data<Arc<Addr<my_actor::MyActor>>>,
    id: web::Path<i32>,
) -> impl Responder {
    let result = match service.ipc_service.get(id.0) {
        Err(e) => Result::error_description(Result::DB_OPERATION_ERROR, &e.to_string()),
        Ok(ipc) => match ipc.clone() {
            None => Result::error(Result::DATA_NOT_FOUND),
            Some(mut db_ipc) => {
                if db_ipc.enable == 1 {
                    Result::error(Result::ALREADY_PUSHING)
                } else {
                    db_ipc.enable = 1;
                    task::spawn(addr.send(my_actor::Ping(db_ipc.id, db_ipc.enable)));
                    let update_time = util::time::current_timestamp();
                    db_ipc.reason = None;
                    db_ipc.update_time = Some(update_time as i64);
                    match service.ipc_service.update(db_ipc) {
                        Ok(_) => Result::success(),
                        Err(e) => {
                            Result::error_description(Result::DB_OPERATION_ERROR, &e.to_string())
                        }
                    }
                }
            }
        },
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap())
}

#[get("/api/ipc/{id}/stop")]
pub async fn ipc_publish_stop(
    service: web::Data<Arc<service::Service>>,
    addr: web::Data<Arc<Addr<my_actor::MyActor>>>,
    id: web::Path<i32>,
) -> impl Responder {
    let result = match service.ipc_service.get(id.0) {
        Err(e) => Result::error_description(Result::DB_OPERATION_ERROR, &e.to_string()),
        Ok(ipc) => match ipc.clone() {
            None => Result::error(Result::DATA_NOT_FOUND),
            Some(mut db_ipc) => {
                if db_ipc.enable == 0 {
                    Result::error(Result::NOT_PUSHING)
                } else {
                    db_ipc.enable = 0;
                    db_ipc.retry_count = 0;
                    db_ipc.reason = None;
                    task::spawn(addr.send(my_actor::Ping(db_ipc.id, db_ipc.enable)));
                    let update_time = util::time::current_timestamp();
                    db_ipc.update_time = Some(update_time as i64);
                    match service.ipc_service.update(db_ipc) {
                        Ok(_) => Result::success(),
                        Err(e) => {
                            Result::error_description(Result::DB_OPERATION_ERROR, &e.to_string())
                        }
                    }
                }
            }
        },
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap())
}

#[delete("/api/ipc/{id}")]
pub async fn delete_ipc(
    service: web::Data<Arc<service::Service>>,
    id: web::Path<i32>,
) -> impl Responder {
    let id = id.0;
    let result = match service.ipc_service.get(id) {
        Err(e) => Result::error_description(Result::DB_OPERATION_ERROR, &e.to_string()),
        Ok(ipc) => match ipc.clone() {
            None => Result::error(Result::DATA_NOT_FOUND),
            Some(db_ipc) => {
                if db_ipc.enable == 1 {
                    Result::error(Result::ALREADY_PUSHING)
                } else {
                    match service.ipc_service.delete(id) {
                        Ok(_) => Result::success(),
                        Err(e) => {
                            Result::error_description(Result::DB_OPERATION_ERROR, &e.to_string())
                        }
                    }
                }
            }
        },
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&result).unwrap())
}

#[get("/api/ipc/{id}")]
pub async fn get_ipc(
    service: web::Data<Arc<service::Service>>,
    id: web::Path<i32>,
) -> impl Responder {
    let id = id.0;
    let result = match service.ipc_service.get(id) {
        Err(e) => serde_json::to_string(&Result::error_description(
            Result::DB_OPERATION_ERROR,
            &e.to_string(),
        )),
        Ok(ipc) => serde_json::to_string(&Result::success_return_data(ipc)),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(result.unwrap())
}

#[get("/api/ipcs")]
pub async fn get_ipc_list(
    service: web::Data<Arc<service::Service>>,
    web::Query(paging): web::Query<PagingInfoReq>,
) -> impl Responder {
    let page = match paging.page {
        None => 1,
        Some(page) => page,
    };
    let rows = match paging.rows {
        None => 10,
        Some(rows) => rows,
    };
    let result = match service.ipc_service.count() {
        Err(e) => serde_json::to_string(&Result::error_description(
            Result::DB_OPERATION_ERROR,
            &e.to_string(),
        )),
        Ok(total) => match service.ipc_service.get_list(page, rows, paging.keyword) {
            Ok(ipc_list) => {
                let page = Page::new(total, ipc_list);
                serde_json::to_string(&Result::success_return_data(page))
            }
            Err(e) => serde_json::to_string(&Result::error_description(
                Result::DB_OPERATION_ERROR,
                &e.to_string(),
            )),
        },
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(result.unwrap())
}

#[get("/api/ipcs/num")]
pub async fn get_ip_num(service: web::Data<Arc<service::Service>>) -> impl Responder {
    let total = match service.ipc_service.count() {
        Err(_) => 0,
        Ok(total) => total,
    };

    let enable_num = match service.ipc_service.count_enable() {
        Err(_) => 0,
        Ok(num) => num,
    };

    let reason_num = match service.ipc_service.count_reason() {
        Err(_) => 0,
        Ok(num) => num,
    };

    let mut map: HashMap<&str, u64> = HashMap::new();
    map.insert("total", total);
    map.insert("enable_num", enable_num);
    map.insert("reason_num", reason_num);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&Result::success_return_data(map)).unwrap())
}

#[get("/api/ipc/key/gen")]
pub async fn gen_key(service: web::Data<Arc<service::Service>>) -> impl Responder {
    let total = match service.ipc_service.count() {
        Err(_) => 0,
        Ok(total) => total,
    } + 1;

    let key = format!("D{:04X}", total);

    let mut map: HashMap<&str, String> = HashMap::new();
    map.insert("key", key);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&Result::success_return_data(map)).unwrap())
}
