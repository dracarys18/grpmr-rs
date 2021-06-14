use crate::db::db_utils::get_userid_from_name;
use crate::{get_mdb, Cxt, TgErr};
use anyhow::anyhow;
use std::str::FromStr;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberKind, ChatMemberStatus, MessageEntity, MessageEntityKind};

pub async fn get_bot_id(cx: &Cxt) -> i64 {
    return cx.requester.get_me().await.unwrap().user.id;
}

pub async fn can_user_restrict(cx: &Cxt, user_id: i64) -> bool {
    let ret = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id)
        .await
        .ok();
    if ret.is_none() {
        return false;
    }
    let mem = ret.unwrap();
    if let ChatMemberKind::Creator(_) = &mem.kind {
        return true;
    }
    if let ChatMemberKind::Administrator(a) = &mem.kind {
        if a.can_restrict_members {
            return true;
        }
    }
    return false;
}
pub async fn user_should_restrict(cx: &Cxt, user_id: i64) -> TgErr<()> {
    if can_user_restrict(cx, user_id).await {
        return Ok(());
    }
    cx.reply_to("I can't restrict people here make sure you gave me proper rights to do so!!")
        .await?;
    return Err(anyhow!("User don't have the permission to restrict"));
}
#[allow(dead_code)]
pub async fn is_user_admin(cx: &Cxt, user_id: i64) -> bool {
    let ret = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id)
        .await
        .ok();

    if ret.is_none() {
        return false;
    }
    let mem = ret.unwrap();
    if matches!(
        mem.status(),
        ChatMemberStatus::Administrator | ChatMemberStatus::Creator
    ) {
        return true;
    }
    return false;
}
#[allow(dead_code)]
pub async fn user_should_be_admin(cx: &Cxt, user_id: i64) -> TgErr<()> {
    if is_user_admin(cx, user_id).await {
        return Ok(());
    }
    cx.reply_to("I am not admin here!").await?;
    return Err(anyhow!("User isnt admin"));
}
pub fn extract_id_from_reply(cx: &Cxt) -> (Option<i64>, Option<String>) {
    let prev_message = cx.update.reply_to_message();
    if prev_message.is_none() {
        return (None, None);
    }
    if let Some(user) = prev_message.unwrap().from() {
        if let Some(msg_text) = prev_message.unwrap().text() {
            let res: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();
            if res.len() < 2 {
                return (Some(user.id), Some("".to_owned()));
            }
            return (Some(user.id), Some(res[1].to_owned()));
        }
        return (Some(user.id), None);
    }
    (None, None)
}
pub async fn extract_text_id_from_reply(cx: &Cxt) -> (Option<i64>, Option<String>) {
    if let Some(msg_text) = cx.update.text() {
        let split_text: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();

        if split_text.len() < 2 {
            return extract_id_from_reply(cx);
        }

        let text_to_parse = split_text[1];
        let args: Vec<_> = text_to_parse.split_whitespace().collect();

        let mut user_id: Option<i64> = None;
        let mut text: Option<String> = None;
        let mut ent: Option<&MessageEntity> = None;

        if let Some(entities) = cx.update.entities() {
            let filtered_entities: Vec<_> = entities
                .iter()
                .filter(|entity| matches!(entity.kind, MessageEntityKind::TextMention { user: _ }))
                .collect();

            if !filtered_entities.is_empty() {
                ent = Some(&entities[0]);
            }

            if !entities.is_empty() && ent.is_some() {
                if ent.unwrap().offset == msg_text.len() - text_to_parse.len() {
                    ent = Some(&entities[0]);
                    user_id = match &ent.unwrap().kind {
                        MessageEntityKind::TextMention { user } => Some(user.id),
                        _ => None,
                    };
                    text = Some(msg_text[ent.unwrap().offset + ent.unwrap().length..].to_owned());
                }
            } else if !args.is_empty() && args[0].starts_with('@') {
                let user_name = args[0];
                let db = get_mdb().await;
                let res = get_userid_from_name(
                    &db,
                    user_name.to_string().replace("@", "").to_lowercase(),
                )
                .await;
                if res.is_ok() {
                    user_id = res.unwrap();
                    let split: Vec<_> = msg_text.splitn(3, char::is_whitespace).collect();
                    if split.len() >= 3 {
                        text = Some(split[2].to_owned());
                    }
                } else {
                    cx.reply_to(
                        "Could not find a user by this name; are you sure I've seen them before?",
                    )
                    .await
                    .ok();
                    return (None, None);
                }
            } else if !args.is_empty() {
                if let Ok(id) = args[0].parse::<i64>() {
                    user_id = Some(id);
                    let res: Vec<_> = msg_text.splitn(3, char::is_whitespace).collect();
                    if res.len() >= 3 {
                        text = Some(res[2].to_owned());
                    }
                }
            } else if cx.update.reply_to_message().is_some() {
                let (id, tet) = extract_id_from_reply(&cx);
                user_id = id;
                text = tet;
            } else {
                return (None, None);
            }

            if let Some(id) = user_id {
                match cx.requester.get_chat(id).await {
                    Ok(_) => {}
                    Err(_) => {
                        cx.reply_to("I don't seem to have interacted with this user before - please forward a message from them to give me control! (like a voodoo doll, I need a piece of them to be able to execute certain commands...)").await.ok();
                        return (None, None);
                    }
                }
            }
        }
        return (user_id, text);
    }
    (None, None)
}

pub async fn is_group(cx: &Cxt) -> TgErr<()> {
    let c = &cx.update.chat;
    if c.is_group() || c.is_supergroup() {
        return Ok(());
    }
    cx.reply_to("This command can't be used in private").await?;
    return Err(anyhow!("This isnt a group"));
}

pub async fn can_send_text(cx: &Cxt, id: i64) -> TgErr<bool> {
    if cx.update.chat.is_private() {
        return Ok(false);
    }
    let mem = cx.requester.get_chat_member(cx.chat_id(), id).await?;
    let restricted = mem.kind.can_send_messages();
    Ok(!restricted)
}

pub async fn is_user_restricted(cx: &Cxt, id: i64) -> anyhow::Result<bool> {
    if cx.update.chat.is_private() {
        return Ok(false);
    }
    let mem = cx.requester.get_chat_member(cx.chat_id(), id).await?;
    let restricted = mem.kind.can_send_messages()
        && mem.kind.can_send_media_messages()
        && mem.kind.can_send_other_messages()
        && mem.kind.can_add_web_page_previews();
    Ok(!restricted)
}

pub async fn can_pin_messages(cx: &Cxt, id: i64) -> TgErr<()> {
    let mem = cx.requester.get_chat_member(cx.chat_id(), id).await?;
    match &mem.kind {
        ChatMemberKind::Creator(_) => {
            return Ok(());
        }
        ChatMemberKind::Administrator(_) => {
            if mem.kind.can_pin_messages() {
                return Ok(());
            }
        }
        _ => {}
    }
    cx.reply_to("Missing CAN_PIN_MESSAGES permissions").await?;
    Err(anyhow!(
        "Can't pin messages because missing can_pin_messages permissions"
    ))
}

pub async fn can_promote_members(cx: &Cxt, id: i64) -> TgErr<()> {
    let mem = cx.requester.get_chat_member(cx.chat_id(), id).await?;
    match &mem.kind {
        ChatMemberKind::Creator(_) => {
            return Ok(());
        }
        ChatMemberKind::Administrator(_) => {
            if mem.kind.can_promote_members() {
                return Ok(());
            }
        }
        _ => {}
    }
    cx.reply_to("Missing CAN_PROMOTE_MEMBERS permissions")
        .await?;
    Err(anyhow!(
        "Can't promote members because user is missing can_promote_members permissions"
    ))
}

pub enum PinMode {
    Loud,
    Silent,
    Error,
}

impl FromStr for PinMode {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let ret = match s {
            "loud" | "hard" | "violent" => PinMode::Loud,
            "silent" => PinMode::Silent,
            _ => PinMode::Error,
        };
        return Ok(ret);
    }
}
