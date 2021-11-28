pub mod account;
pub mod ipc;
pub mod start;

use log::info;

#[derive(Clone)]
pub struct Service {
    pub ipc_service: ipc::IpcService,
    pub account_service: account::AccountService,
}

impl Service {
    pub fn new() -> Self {
        let ipc_service = match ipc::IpcService::new() {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
        let account_service = match account::AccountService::new() {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
        match account_service.init_data() {
            Err(e) => panic!("{}", e),
            Ok(ret) => {
                if ret > 0 {
                    info!("init account successs");
                }
            }
        };
        Service {
            ipc_service,
            account_service,
        }
    }
}
