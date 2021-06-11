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
