use super::commands::Command;
use crate::teloxide::utils::command::BotCommand;
use crate::{Cxt, Err};
use teloxide::utils::html::user_mention_or_link;

pub async fn start_handler(cx: &Cxt) -> Err {
    let start_message_priv = format!(
        "Hello {}! Hope you are doing well\n Send /help to know about available commands",
        user_mention_or_link(cx.update.from().unwrap())
    );
    let start_message_pub = "I am alive boss!".to_owned();

    if cx.update.chat.is_private() {
        cx.reply_to(start_message_priv).await?;
        return Ok(());
    }
    cx.reply_to(start_message_pub).await?;
    return Ok(());
}

pub async fn help_handler(cx: &Cxt) -> Err {
    let descriptions = Command::descriptions();
    if cx.update.chat.is_group() || cx.update.chat.is_supergroup() {
        cx.reply_to("This command is meant to be used in private")
            .await?;
        return Ok(());
    }
    cx.reply_to(descriptions).await?;
    return Ok(());
}
