use crate::database::db_utils::get_log_channel;
use crate::modules::send_log;
use crate::util::{can_delete_messages, get_bot_id, get_chat_title, is_group};
use crate::{get_mdb, Cxt, TgErr};
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::html;
use tokio::time::Duration;
pub async fn delete(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        can_delete_messages(cx, get_bot_id(cx).await),
        can_delete_messages(cx, cx.update.from().unwrap().id),
    )?;
    if let Some(msg) = cx.update.reply_to_message() {
        let msg_id = msg.id;
        if let Err(m) = cx.requester.delete_message(cx.chat_id(), msg_id).await {
            cx.reply_to(format!(
                "Can't delete the message for the following reason\n<code>{}</code>",
                m
            ))
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    } else {
        cx.reply_to("Reply to some message to delete it").await?;
        return Ok(());
    }
    if let Err(m) = cx
        .requester
        .delete_message(cx.chat_id(), cx.update.id)
        .await
    {
        cx.reply_to(format!(
            "Can't delete the message for the following reason<code>{}</code>",
            m
        ))
        .parse_mode(ParseMode::Html)
        .await?;
    }
    Ok(())
}
pub async fn purge(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        can_delete_messages(cx, get_bot_id(cx).await),
        can_delete_messages(cx, cx.update.from().unwrap().id),
    )?;
    let db = get_mdb().await;
    let mut count: u32 = 0;
    if let Some(msg) = cx.update.reply_to_message() {
        let msg_id = msg.id;
        let delete_to = cx.update.id;
        for m_id in (msg_id..delete_to).rev() {
            if let Err(m) = cx.requester.delete_message(cx.chat_id(), m_id).await {
                cx.reply_to(format!("Error in purging some of the messages reason message might be too old or this is not a supergroup\n Error Message: <code>{}</code>",
                m
            ))
                .parse_mode(ParseMode::Html)
                .await?;
                return Ok(());
            }
            count += 1;
        }
        if let Err(m) = cx
            .requester
            .delete_message(cx.chat_id(), cx.update.id)
            .await
        {
            cx.requester
                .send_message(
                    cx.chat_id(),
                    format!(
                        "Error while deleting messages\n Error Message:<code>{}</code>",
                        m
                    ),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            return Ok(());
        }
    } else {
        cx.reply_to("Reply to some message to purge").await?;
        return Ok(());
    }
    let msg = cx
        .requester
        .send_message(cx.chat_id(), format!("Purged {} messages", count))
        .await?;
    if let Some(l) = get_log_channel(&db, cx.chat_id()).await? {
        let admin = cx
            .requester
            .get_chat_member(cx.chat_id(), cx.update.from().unwrap().id)
            .await?
            .user;
        let logm = format!(
            "Chat title: {}\n#PURGE\nAdmin: {}\nNo of messages: {}",
            html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
            html::user_mention(admin.id, &admin.full_name()),
            html::code_inline(&count.to_string())
        );
        send_log(cx, &logm, l).await?;
    }
    tokio::time::sleep(Duration::from_secs(4)).await;
    cx.requester.delete_message(cx.chat_id(), msg.id).await?;
    Ok(())
}
