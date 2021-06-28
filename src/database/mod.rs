pub mod db;
pub mod db_utils;

pub use db::Db;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub user_name: String,
}
#[derive(Serialize, Deserialize)]
pub struct Chat {
    pub chat_id: i64,
    pub chat_name: String,
}
#[derive(Serialize, Deserialize)]
pub struct Gban {
    pub user_id: i64,
    pub reason: String,
}
#[derive(Serialize, Deserialize)]
pub struct GbanStat {
    pub chat_id: i64,
    pub is_on: bool,
}
#[derive(Serialize, Deserialize)]
pub struct Warn {
    pub chat_id: i64,
    pub user_id: i64,
    pub reason: String,
    pub count: u64,
}
#[derive(Serialize, Deserialize)]
pub struct WarnKind {
    pub chat_id: i64,
    pub softwarn: bool,
}
#[derive(Serialize, Deserialize)]
pub struct Warnlimit {
    pub chat_id: i64,
    pub limit: u64,
}
#[derive(Serialize, Deserialize)]
pub struct DisableCommand {
    pub chat_id: i64,
    pub disabled_commands: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Filters {
    pub chat_id: i64,
    pub filter: String,
    pub reply: String,
    pub f_type: String,
}
#[derive(Serialize, Deserialize)]
pub struct BlacklistFilter {
    pub chat_id: i64,
    pub filter: String,
}
#[derive(Serialize, Deserialize)]
pub struct BlacklistKind {
    pub chat_id: i64,
    pub kind: String,
}
