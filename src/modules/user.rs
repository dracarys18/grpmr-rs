use crate::util::extract_text_id_from_reply;
use crate::{Cxt, TgErr, OWNER_ID, SUDO_USERS};
use teloxide::prelude::*;
use teloxide::types::{ChatKind, ForwardedFrom, ParseMode};
use teloxide::utils::command::parse_command;
use teloxide::utils::html::user_mention;

pub async fn info(cx: &Cxt) -> TgErr<()> {
    let (user_id, _) = extract_text_id_from_reply(cx).await;
    let (_, args) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
    let chat = cx.update.chat.clone();
    let mut user = None;
    if user_id.is_some() && (chat.is_group() || chat.is_supergroup()) {
        user = match cx
            .requester
            .get_chat_member(cx.chat_id(), user_id.unwrap())
            .await
        {
            Ok(chatmem) => Some(chatmem.user),
            Err(_) => None,
        };
    } else if (chat.is_private()) || (cx.update.reply_to_message().is_none() && args.is_empty()) {
        user = Some(cx.update.from().unwrap().to_owned());
    } else if user_id.is_none() {
        cx.reply_to("This user is either dead or not in my database!")
            .await?;
        return Ok(());
    }
    if user.is_none() {
        cx.reply_to("Can't seem to get info about the user").await?;
        return Ok(());
    }

    let us_inf = user.unwrap();
    let mut info_text = format!(
        "<b>User info</b>:\nID:<code>{}</code>\nFirst Name: {}",
        &us_inf.id, &us_inf.first_name
    );

    if let Some(ln) = us_inf.clone().last_name {
        info_text = format!("{}\nLast Name: {}", info_text, ln);
    }

    if let Some(un) = us_inf.clone().username {
        info_text = format!("{}\nUsername: @{}", info_text, un);
    }

    info_text = format!(
        "{}\nPermanent Link: {}",
        info_text,
        user_mention(us_inf.id as i32, "link")
    );

    if us_inf.id == *OWNER_ID {
        info_text = format!(
            "{}\n\n<i>Note:-This user is my owner I will always be loyal to him</i>",
            info_text
        );
    } else if (*SUDO_USERS).contains(&us_inf.id) {
        info_text = format!(
            "{}\n\n<i>Note:-This is one of my sudo users as powerful as my owner beware</i>",
            info_text
        );
    }
    cx.reply_to(info_text).parse_mode(ParseMode::Html).await?;
    Ok(())
}

pub async fn get_id(cx: &Cxt) -> TgErr<()> {
    let (user_id, _) = extract_text_id_from_reply(cx).await;
    if user_id.is_some() {
        if let Some(msg) = cx.update.reply_to_message() {
            let user = msg.from();
            if let Some(frwd) = msg.forward_from() {
                let us1 = user.unwrap();
                if let ForwardedFrom::User(us) = frwd {
                    cx.reply_to(format!(
                        "The sender {} has ID <code>{}</code> and the forwarder {} has ID <code>{}</code>",
                        user_mention(us.id as i32,&us.first_name),
                        us.id,
                        user_mention(us1.id as i32,&us.first_name),
                        us1.id))
                        .parse_mode(ParseMode::Html)
                        .await?;
                } else if let ForwardedFrom::SenderName(_) = frwd {
                    cx.reply_to(format!(
                        "{}'s ID is <code>{}</code>",
                        user_mention(us1.id as i32, &us1.first_name),
                        us1.id
                    ))
                    .parse_mode(ParseMode::Html)
                    .await?;
                }
            } else if let Some(u) = user {
                cx.reply_to(format!(
                    "{}'s ID is <code>{}</code>",
                    user_mention(u.id as i32, &u.first_name),
                    u.id
                ))
                .parse_mode(ParseMode::Html)
                .await?;
            } else {
                cx.reply_to("This user's dead I can't get his ID").await?;
            }
        } else if let ChatKind::Private(u) = cx.requester.get_chat(user_id.unwrap()).await?.kind {
            cx.reply_to(format!(
                "{}'s ID is <code>{}</code>",
                user_mention(
                    user_id.unwrap() as i32,
                    &u.first_name.unwrap_or_else(|| "".to_string())
                ),
                user_id.unwrap()
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        }
    } else {
        let c = &cx.update.chat;
        cx.reply_to(format!("This chat has ID of <code>{}</code>", c.id))
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}
