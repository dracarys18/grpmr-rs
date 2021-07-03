use teloxide::{
    payloads::SendMessageSetters,
    prelude::{GetChatId, Requester},
    types::ParseMode,
    utils::html,
};

use crate::{
    database::{
        db_utils::{add_log_channel, get_log_channel, rm_log_channel},
        Logging,
    },
    get_mdb,
    util::{get_bot_id, is_group, user_should_be_creator},
    Cxt, TgErr,
};

pub async fn add_logc(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_creator(cx, cx.update.from().unwrap().id)
    )?;
    let bot_id = get_bot_id(cx).await;
    let db = get_mdb().await;
    if cx.update.reply_to_message().is_none() {
        cx.reply_to("Add the bot to the channel send a message in the channel and forward that message to this chat and reply to the message with /setlog").await?;
        return Ok(());
    }
    let ch_msg = cx.update.reply_to_message().unwrap();
    if let Some(m) = ch_msg.forward_from_chat() {
        if let Ok(chatmem) = cx.requester.get_chat_member(m.id, bot_id).await {
            let c_id = m.id;
            if chatmem.can_post_messages() {
                let lg = Logging {
                    chat_id: cx.chat_id(),
                    channel: c_id,
                };
                add_log_channel(&db, &lg).await?;
                cx.reply_to(format!(
                    "Added log channel for this chat\nThe channel id of the log channel is:{}",
                    html::code_inline(&c_id.to_string())
                ))
                .parse_mode(ParseMode::Html)
                .await?;
            } else {
                cx.reply_to("I can't post messages in the mentioned channel please give me permissions to do so").await?;
            }
        } else {
            cx.reply_to(
                "Seems like I am not in the channel where the message was forwardeed from!",
            )
            .await?;
        }
    } else {
        cx.reply_to("This message was not forwarded from a channel please read the instruction properly for setting a log channel").await?;
    }
    Ok(())
}

pub async fn remove_log(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_creator(cx, cx.update.from().unwrap().id)
    )?;
    let db = get_mdb().await;
    if let Some(_) = get_log_channel(&db, cx.chat_id()).await? {
        rm_log_channel(&db, cx.chat_id()).await?;
        cx.reply_to("The log channel has been unset").await?;
    } else {
        cx.reply_to("You haven't set any log channel's here send /setlog to get instructions")
            .await?;
    }
    Ok(())
}

pub async fn send_log(cx: &Cxt, message: &str, channel_id: i64) -> TgErr<()> {
    if let Err(e) = cx
        .requester
        .send_message(channel_id, message)
        .parse_mode(ParseMode::Html)
        .await
    {
        let text = format!(
            "Can't send the message into the log channel because of\n{}",
            html::code_inline(&e.to_string())
        );
        cx.requester
            .send_message(cx.chat_id(), text)
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}
