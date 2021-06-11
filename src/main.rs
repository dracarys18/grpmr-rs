extern crate teloxide;
mod commands;
mod db;
mod util;
use async_once::AsyncOnce;
use commands::admin::*;
use commands::start::*;
use commands::Command;
use db::db_utils::{save_chat, save_user};
use db::Db;
use dotenv;
use lazy_static::lazy_static;
use std::error::Error;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand as Cmd;

pub type Cxt = UpdateWithCx<AutoSend<Bot>, Message>;
pub type Err = Result<(), Box<dyn Error + Send + Sync>>;

lazy_static! {
    pub static ref MONGO_URI: String = dotenv::var("MONGO_URI").expect("MONGO_URI is not defined");
    pub static ref MDB: AsyncOnce<mongodb::Database> =
        AsyncOnce::new(async { Db::new(MONGO_URI.to_owned()).client().await });
}
async fn get_mdb() -> mongodb::Database {
    MDB.get().await.clone()
}
async fn answer(cx: Cxt) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mngdb = get_mdb().await;
    tokio::try_join!(save_user(&cx, &mngdb), save_chat(&cx, &mngdb))?;
    let txt = cx.update.text();
    if txt.is_none() {
        return Ok(());
    }
    let command = Cmd::parse(txt.unwrap(), "Tgbot-RS");
    if let Ok(c) = command {
        match c {
            Command::Ban => {
                ban(&cx).await?;
            }
            Command::Unban => unban(&cx).await?,
            Command::Mute => mute(&cx).await?,
            Command::Unmute => unmute(&cx).await?,
            Command::Start => start_handler(&cx).await?,
            Command::Help => help_handler(&cx).await?,
        }
    }
    Ok(())
}
async fn run() {
    dotenv::dotenv().ok();
    teloxide::enable_logging!();
    let bot = Bot::from_env().auto_send();
    log::info!("Bot started");
    teloxide::repl(bot.clone(), answer).await;
}
#[tokio::main]
async fn main() {
    run().await;
}
