pub mod auth;
pub mod backup_job_list;
pub mod backup_job_put;

use pluto_network::{ topics::Topic, handler::Handler };
use std::{ sync::Arc, collections::HashMap };

macro_rules! __use_handlers {
    ($m:ident, $($h:expr),* $(,)?) => {
        $(
            $m.insert($h.topic(), Arc::new($h));
        )*
    }
}

lazy_static::lazy_static! {
    pub static ref HANDLERS: HashMap<Topic, Arc<dyn Handler>> = {
        let mut h: HashMap<Topic, Arc<dyn Handler>> = HashMap::new();

        __use_handlers! { h,
            // register implemented message handlers here
            auth::AuthHandler,
            backup_job_list::BackupJobListHandler,
            backup_job_put::BackupJobPutHandler
        }

        h
    };
}
