pub mod db;
pub mod db_utils;
pub use db::Db;

pub struct User {
    pub user_id: i64,
    pub user_name: String,
}
pub struct Chat {
    pub chat_id: i64,
    pub chat_name: String,
}

pub struct Gban {
    pub user_id: i64,
    pub reason: String,
}

pub struct Warn {
    pub chat_id: i64,
    pub user_id: i64,
    pub reason: String,
    pub count: u64,
}

pub struct Warnlimit {
    pub chat_id: i64,
    pub limit: u64,
}
