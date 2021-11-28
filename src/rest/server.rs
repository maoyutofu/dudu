// use actix_files as fs;
use actix::prelude::*;
use actix_session::CookieSession;
use actix_web::{App, HttpServer};

use super::account;
use super::auth;
use super::index;
use super::ipc;
use super::login;
use crate::config::Config;
use crate::my_actor;
use crate::service;
use async_std::task;
use std::io;
use std::sync::Arc;

use log::info;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Server {}
    }

    pub async fn start(&self) -> io::Result<()> {
        let config = Config::new();
        let bind = format!("{}:{}", config.http.host, config.http.port);

        info!("Listening on http://{}", bind);

        let addr = my_actor::MyActor::new().start();
        let addr_arc = Arc::new(addr);

        let service = service::Service::new();
        let service_arc = Arc::new(service);

        // 启动上一次非正常结束的推流任务
        task::spawn(service::start::start_undone(
            service_arc.clone(),
            addr_arc.clone(),
        ));
        // 定时检查 异常终止任务，并重试
        task::spawn(service::start::retry_abnormal(
            config,
            service_arc.clone(),
            addr_arc.clone(),
        ));

        // 定时检查 状态检查
        task::spawn(service::start::status_check(service_arc.clone()));

        HttpServer::new(move || {
            App::new()
                .wrap(CookieSession::signed(&[0; 32]).secure(false))
                .wrap(auth::Auth(service_arc.clone()))
                .data(service_arc.clone())
                .data(addr_arc.clone())
                .service(index::hello)
                .service(login::login)
                .service(ipc::add_ipc)
                .service(ipc::update_ipc)
                .service(ipc::delete_ipc)
                .service(ipc::get_ipc_list)
                .service(ipc::get_ipc)
                .service(ipc::ipc_publish_start)
                .service(ipc::ipc_publish_stop)
                .service(ipc::get_ip_num)
                .service(ipc::gen_key)
                .service(account::change_password)
            // .service(fs::Files::new("/admin", "./public").index_file("default.html"))
        })
        .bind(bind)?
        .run()
        .await
    }
}
