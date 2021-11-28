use super::Service;
use crate::config::Config;
use crate::my_actor;

use actix::prelude::*;
use async_std::task;
use std::sync::Arc;
use std::thread;
use std::time;

use log::{error, info};

/// 检查上次程序结束，是否存在未完成任务
/// 如果存在就启动该任务
pub async fn start_undone(service: Arc<Service>, addr: Arc<Addr<my_actor::MyActor>>) {
    info!("{}", "Start unfinished tasks");
    match service.ipc_service.get_enable_list() {
        Err(e) => panic!("{}", e),
        Ok(ipc_list) => {
            for ipc in ipc_list.iter() {
                // 启动推流任务
                task::spawn(addr.send(my_actor::Ping(ipc.id, ipc.enable)));
            }
        }
    }
}

/// 检测是否存在异常任务
/// 如果存在就重启该任务
pub async fn retry_abnormal(
    config: Config,
    service: Arc<Service>,
    addr: Arc<Addr<my_actor::MyActor>>,
) {
    // 方法延后启动时间
    let delay_time = time::Duration::from_millis(60000);
    thread::sleep(delay_time);

    let less_retry_count = config.publisher.max_retry_count;
    let interval_time = time::Duration::from_millis(config.publisher.interval_time);
    let task_interval_time = time::Duration::from_millis(config.publisher.task_interval_time);
    loop {
        info!("{}", "Retry abnormal task");
        match service.ipc_service.get_list_by_reason(less_retry_count) {
            Err(e) => panic!("{}", e),
            Ok(ipc_list) => {
                for ipc in ipc_list.iter() {
                    info!("Retry abnormal task: {}", ipc.name);
                    let mut ipc_clone = ipc.clone();
                    // 数据库增加一次重试次数
                    ipc_clone.retry_count += 1;
                    ipc_clone.enable = 1;
                    ipc_clone.reason = None;
                    match service.ipc_service.update(ipc_clone) {
                        Ok(_) => {
                            // 启动推流任务
                            task::spawn(addr.send(my_actor::Ping(ipc.id, 1)));
                        }
                        Err(e) => error!("{}", &e.to_string()),
                    }
                    thread::sleep(task_interval_time); // 每个任务间隔时间
                }
            }
        }
        thread::sleep(interval_time); // 定时查询时间
    }
}

/// 状态检查
/// 1、将`enable=1`的`retry_count`重置为0
pub async fn status_check(service: Arc<Service>) {
    // 方法延后启动时间
    let delay_time = time::Duration::from_millis(30000);

    loop {
        thread::sleep(delay_time);
        info!("{}", "Status check task");
        match service.ipc_service.get_enable_list() {
            Err(e) => panic!("{}", e),
            Ok(ipc_list) => {
                for ipc in ipc_list.iter() {
                    let mut ipc_clone = ipc.clone();
                    ipc_clone.retry_count = 0;
                    ipc_clone.reason = None;

                    match service.ipc_service.update(ipc_clone) {
                        Ok(_) => {}
                        Err(e) => error!("{}", &e.to_string()),
                    }
                }
            }
        }
    }
}
