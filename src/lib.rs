//! 一个基于 Rust语言 的推流器，支持 IP Caream 和本地视频文件推送到流媒体服务器。  
//! 本推流器使用 rusty_ffmpeg 调用 ffmpeg 来实现推流，采用 actix 提供 HTTP 接口，数据存储在 sqlite。  

// 数据库连接
pub mod db;

// actor
pub mod my_actor;

// 推流器核心部分
pub mod publisher;

// 通用的 HTTP 接口返回
pub mod result;

// 对数据库操作的总入口
// 主要是系统账号和IPC信息维护相关
pub mod service;

// 工具相关
pub mod util;

// rest 接口相关
pub mod rest;

pub mod config;
