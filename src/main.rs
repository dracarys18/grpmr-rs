extern crate teloxide;
mod database;
mod modules;
mod util;
use crate::util::{consts, enforce_gban};
use async_once::AsyncOnce;
use database::db_utils::{save_chat, save_user};
use database::Db;
use lazy_static::lazy_static;
use modules::*;
use regex::Regex;
use std::error::Error;
use teloxide::prelude::*;
use teloxide::utils::command::{parse_command, BotCommand as Cmd};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub type Cxt = UpdateWithCx<AutoSend<Bot>, Message>;
pub type Ctx = UpdateWithCx<AutoSend<Bot>, CallbackQuery>;
pub type TgErr<T> = anyhow::Result<T>;

lazy_static! {
    pub static ref MONGO_URI: String = dotenv::var("MONGO_URI").expect("MONGO_URI is not defined");
    pub static ref OWNER_ID: i64 = dotenv::var("OWNER_ID")
        .expect("OWNER_ID is not defined")
        .parse::<i64>()
        .unwrap_or(0);
    pub static ref BOT_TOKEN: String =
        dotenv::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN is empty");
    pub static ref SUDO_USERS: Vec<i64> = dotenv::var("SUDO_USERS")
        .expect("SUDO_USERS is not defined")
        .split(',')
        .map(|s| s.parse::<i64>().unwrap_or(0))
        .collect::<Vec<i64>>();
    pub static ref MDB: AsyncOnce<mongodb::Database> =
        AsyncOnce::new(async { Db::new(MONGO_URI.to_owned()).client().await });
}
async fn get_mdb() -> mongodb::Database {
    MDB.get().await.clone()
}
async fn answer(cx: Cxt) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mngdb = get_mdb().await;
    tokio::try_join!(save_user(&cx, &mngdb), save_chat(&cx, &mngdb))?;

    /*
    Gban and Chat Triggers
    */
    if cx.update.chat.is_group() || cx.update.chat.is_supergroup() {
        tokio::try_join!(enforce_gban(&cx))?;
        action_blacklist(&cx).await?;
        filter_reply(&cx).await?;
    }
    let txt = cx.update.text();
    if txt.is_none() {
        return Ok(());
    }
    let command = Cmd::parse(txt.unwrap(), consts::BOT_NAME);
    if let Ok(c) = command {
        let (cmnd, _) = parse_command(txt.unwrap(), consts::BOT_NAME).unwrap();
        match c {
            Command::Ban => {
                ban(&cx).await?;
            }
            Command::Tban => temp_ban(&cx).await?,
            Command::Unban => unban(&cx).await?,
            Command::Mute => mute(&cx).await?,
            Command::Tmute => temp_mute(&cx).await?,
            Command::Unmute => unmute(&cx).await?,
            Command::Start => start_handler(&cx, cmnd).await?,
            Command::Help => help_handler(&cx).await?,
            Command::Kick => kick(&cx).await?,
            Command::Kickme => kickme(&cx, cmnd).await?,
            Command::Info => info(&cx, cmnd).await?,
            Command::Id => get_id(&cx).await?,
            Command::Pin => pin(&cx).await?,
            Command::Unpin => unpin(&cx).await?,
            Command::Promote => promote(&cx).await?,
            Command::Demote => demote(&cx).await?,
            Command::Invitelink => invitelink(&cx).await?,
            Command::Adminlist => adminlist(&cx, cmnd).await?,
            Command::Purge => purge(&cx).await?,
            Command::Del => delete(&cx).await?,
            Command::Leavechat => leavechat(&cx).await?,
            Command::Ud => ud(&cx, cmnd).await?,
            Command::Paste => katbin(&cx, cmnd).await?,
            Command::Lock => lock(&cx).await?,
            Command::Unlock => unlock(&cx).await?,
            Command::Locktypes => locktypes(&cx).await?,
            Command::Chatlist => chatlist(&cx).await?,
            Command::Gban => gban(&cx).await?,
            Command::Ungban => ungban(&cx).await?,
            Command::Gbanstat => gbanstat(&cx).await?,
            Command::Warn => warn(&cx).await?,
            Command::Warnlimit => warn_limit(&cx).await?,
            Command::Warnmode => warnmode(&cx).await?,
            Command::Resetwarns => reset_warns(&cx).await?,
            Command::Warns => warns(&cx).await?,
            Command::Disable => disable(&cx).await?,
            Command::Enable => enable(&cx).await?,
            Command::Filter => filter(&cx).await?,
            Command::Filters => filter_list(&cx).await?,
            Command::Stop => remove_filter(&cx).await?,
            Command::Addblacklist => blacklist_filter(&cx).await?,
            Command::Blacklists => list_blacklist(&cx).await?,
            Command::Rmblacklist => remove_blacklist(&cx).await?,
            Command::Blacklistmode => set_blacklist_kind(&cx).await?,
            Command::Setchatpic => set_chatpic(&cx).await?,
            Command::Setchattitle => set_chat_tile(&cx).await?,
            Command::Setlog => add_logc(&cx).await?,
            Command::Unsetlog => remove_log(&cx).await?,
            Command::Report => report(&cx).await?,
            Command::Reports => report_set(&cx).await?,
        }
    }
    Ok(())
}
async fn answer_callback(cx: Ctx) -> TgErr<()> {
    let data = &cx.update.data;
    if let Some(d) = data {
        let warn_re = Regex::new(r#"rm_warn\((.+?)\)"#).unwrap();
        if warn_re.is_match(&d) {
            handle_unwarn_button(&cx).await?;
        }
    }
    Ok(())
}
async fn run() {
    dotenv::dotenv().ok();
    teloxide::enable_logging!();
    let bot = Bot::from_env().auto_send();
    log::info!("Bot started");
    Dispatcher::new(bot)
        .messages_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |cx| async move {
                answer(cx).await.log_on_error().await
            })
        })
        .callback_queries_handler(|rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |cx| async move {
                answer_callback(cx).await.log_on_error().await
            })
        })
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}
#[tokio::main]
async fn main() {
    run().await;
}
