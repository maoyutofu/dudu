use super::publisher;
use super::service;
use super::util;
use actix::prelude::*;
use async_std::task;
use log::{error, info};
use std::sync::Arc;

pub struct MyActor {
    pub publisher_list: Vec<Arc<publisher::Publisher>>,
    pub service: Arc<service::Service>,
}
impl MyActor {
    pub fn new() -> Self {
        MyActor {
            publisher_list: Vec::new(),
            service: Arc::new(service::Service::new()),
        }
    }
    pub fn get_index(&self, id: i32) -> Option<usize> {
        self.publisher_list.iter().position(|x| x.id == id)
    }
}

impl Actor for MyActor {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "i32")]
pub struct Ping(pub i32, pub i32);

impl Handler<Ping> for MyActor {
    type Result = i32;

    fn handle(&mut self, msg: Ping, _ctx: &mut Context<Self>) -> Self::Result {
        let id = msg.0;
        let ipc_result = self.service.ipc_service.get(id);
        if ipc_result.is_err() {
            error!("{}", ipc_result.unwrap_err());
            return -1;
        }
        let option_ipc = ipc_result.unwrap();
        if option_ipc.is_none() {
            error!("id: {} not found", id);
            return -2;
        }
        let ipc = option_ipc.unwrap();
        if msg.1 == 0 {
            if self.publisher_list.len() > 0 {
                match self.get_index(id) {
                    None => {}
                    Some(index) => {
                        let cmd = &self.publisher_list[index];
                        if cmd.stop() == true {
                            self.publisher_list.remove(index);
                        }
                    }
                };
            }
        } else if msg.1 == 1 {
            let rtsp = ipc.rtsp;
            let rtmp = ipc.rtmp;
            let cmd = publisher::Publisher::new(id);
            let cmd_arc = Arc::new(cmd);
            self.publisher_list.push(cmd_arc.clone());
            let cmd_arc_clone = cmd_arc.clone();
            let service_arc = Arc::clone(&self.service);
            task::spawn(async move {
                let start_result = cmd_arc_clone.start(&rtsp, &rtmp).await;
                match service_arc.ipc_service.get(id) {
                    Err(e) => {
                        error!("{}", &e.to_string());
                    }
                    Ok(option_ipc) => match option_ipc.clone() {
                        None => {}
                        Some(mut db_ipc) => {
                            db_ipc.enable = 0;
                            let update_time = util::time::current_timestamp();
                            db_ipc.update_time = Some(update_time as i64);
                            if start_result.is_ok() {
                                db_ipc.reason = None;
                                db_ipc.retry_count = 0;
                            } else {
                                db_ipc.reason = Some(start_result.unwrap_err());
                            }
                            match service_arc.ipc_service.update(db_ipc) {
                                Ok(_) => {}
                                Err(e) => error!("{}", &e.to_string()),
                            }
                        }
                    },
                }
            });
        } else {
            info!("id: {} {}", msg.0, msg.1);
        }
        msg.0
    }
}
