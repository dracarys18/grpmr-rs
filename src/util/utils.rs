use super::custom_types::*;
use crate::database::db_utils::{
    get_disabled_command, get_gbanstat, get_userid_from_name, is_gbanned,
};
use crate::{get_mdb, Cxt, TgErr, OWNER_ID, SUDO_USERS};
use anyhow::anyhow;
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
    if user_id == *OWNER_ID || (*SUDO_USERS).contains(&user_id) {
        return true;
    }
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
    false
}
pub async fn user_should_restrict(cx: &Cxt, user_id: i64) -> TgErr<()> {
    if can_user_restrict(cx, user_id).await {
        return Ok(());
    }
    if user_id == get_bot_id(cx).await {
        cx.reply_to("I can't restrict people here make sure you gave me proper rights to do so!!")
            .await?;
    } else {
        cx.reply_to("You can't restrict people here!!").await?;
    }
    Err(anyhow!("User don't have the permission to restrict"))
}
pub async fn is_user_admin(cx: &Cxt, user_id: i64) -> bool {
    let ret = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id)
        .await
        .ok();

    if user_id == *OWNER_ID || (*SUDO_USERS).contains(&user_id) {
        return true;
    }
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
    false
}
pub async fn user_should_be_admin(cx: &Cxt, user_id: i64) -> TgErr<()> {
    if is_user_admin(cx, user_id).await {
        return Ok(());
    }
    if user_id == get_bot_id(cx).await {
        cx.reply_to("I am not admin here!").await?;
    } else {
        cx.reply_to("You are not an admin here!").await?;
    }
    Err(anyhow!("User isnt admin"))
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
            return (Some(user.id), Some("".to_owned()));
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
                } else if cx.update.reply_to_message().is_some() {
                    if let Some(u) = cx.update.reply_to_message().unwrap().from() {
                        user_id = Some(u.id);
                    }
                    let res: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();
                    if res.len() >= 2 {
                        text = Some(res[1].to_owned());
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
    Err(anyhow!("This isnt a group"))
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
    if id == *OWNER_ID || (*SUDO_USERS).contains(&id) {
        return Ok(());
    }
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
    if id == get_bot_id(cx).await {
        cx.reply_to("I can't pin messages here! Missing can_pin_messages permission")
            .await?;
    } else {
        cx.reply_to("You can't pin messages here! Missing can_pin_messages permissions")
            .await?;
    }
    Err(anyhow!(
        "Can't pin messages because missing can_pin_messages permissions"
    ))
}

pub async fn can_delete_messages(cx: &Cxt, id: i64) -> TgErr<()> {
    let mem = cx.requester.get_chat_member(cx.chat_id(), id).await?;
    if id == *OWNER_ID || (*SUDO_USERS).contains(&id) {
        return Ok(());
    }
    match &mem.kind {
        ChatMemberKind::Creator(_) => {
            return Ok(());
        }
        ChatMemberKind::Administrator(_) => {
            if mem.kind.can_delete_messages() {
                return Ok(());
            }
        }
        _ => {}
    };
    if id == get_bot_id(cx).await {
        cx.reply_to("I can't delete messages here! Missing can_delete_messages permission")
            .await?;
    } else {
        cx.reply_to("You can't delete messages here! Missing can_delete_messages permission")
            .await?;
    }
    Err(anyhow!(
        "Can't delete messages missing can_delete_messages permission"
    ))
}
pub async fn can_promote_members(cx: &Cxt, id: i64) -> TgErr<()> {
    let mem = cx.requester.get_chat_member(cx.chat_id(), id).await?;
    if id == *OWNER_ID || (*SUDO_USERS).contains(&id) {
        return Ok(());
    }
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
    if id == get_bot_id(cx).await {
        cx.reply_to("I can't promote members here! Missing can_promote_members permission")
            .await?;
    } else {
        cx.reply_to("You can't promote members here! Missing can_promote_members permission")
            .await?;
    }
    Err(anyhow!(
        "Can't promote members because user is missing can_promote_members permissions"
    ))
}

pub async fn owner_filter(uid: i64) -> TgErr<()> {
    if uid == *OWNER_ID {
        return Ok(());
    }
    Err(anyhow!("This command only works for owner"))
}

pub async fn sudo_or_owner_filter(uid: i64) -> TgErr<()> {
    if (*SUDO_USERS).contains(&uid) || *OWNER_ID == uid {
        return Ok(());
    }
    Err(anyhow!(
        "This command only works for either owner or sudo users"
    ))
}

pub async fn check_and_gban(cx: &Cxt, user_id: i64) -> TgErr<()> {
    let db = get_mdb().await;
    if is_gbanned(&db, &user_id).await? && get_gbanstat(&db, cx.chat_id()).await? {
        if cx
            .requester
            .kick_chat_member(cx.chat_id(), user_id)
            .await
            .is_err()
        {
            return Ok(());
        }
        cx.requester.send_message(cx.chat_id(),"This is a Gbanned user trying to sneak inside in my presence. I am banning him right away!").await?;
    }
    Ok(())
}

pub async fn enforce_gban(cx: &Cxt) -> TgErr<()> {
    if let Some(u) = cx.update.from() {
        check_and_gban(cx, u.id).await?;
    } else if let Some(new) = cx.update.new_chat_members() {
        for u in new {
            check_and_gban(cx, u.id).await?;
        }
    }
    Ok(())
}

pub fn get_time(unit: &TimeUnit) -> u64 {
    match unit {
        TimeUnit::Hours(t) => t * 3600,
        TimeUnit::Minutes(t) => t * 60,
        TimeUnit::Seconds(t) => *t,
        TimeUnit::Days(t) => t * 3600 * 24,
    }
}

pub async fn is_command_disabled(chat_id: i64, cmd: String) -> bool {
    if !matches!(cmd.parse::<DisableAble>().unwrap(), DisableAble::Error) {
        let db = get_mdb().await;
        let dis_cmds = get_disabled_command(&db, chat_id).await.unwrap();
        let ind = dis_cmds.iter().position(|p| p.eq(&cmd.to_lowercase()));
        return ind.is_some();
    }
    false
}

pub async fn check_command_disabled(cx: &Cxt, cmd: String) -> TgErr<()> {
    if cx.update.chat.is_group() || cx.update.chat.is_supergroup() {
        if !is_command_disabled(cx.chat_id(), cmd).await {
            return Ok(());
        } else {
            return Err(anyhow!("This command is disabled"));
        }
    }
    Ok(())
}

pub async fn get_filter_type(msg: &Message) -> String {
    if msg.audio().is_some() {
        return "audio".to_string();
    } else if msg.text().is_some() {
        return "text".to_string();
    } else if msg.document().is_some() {
        return "document".to_string();
    } else if msg.photo().is_some() {
        return "photo".to_string();
    } else if msg.video().is_some() {
        return "video".to_string();
    } else if msg.animation().is_some() {
        return "animation".to_string();
    } else if msg.voice().is_some() {
        return "voice".to_string();
    } else if msg.sticker().is_some() {
        return "sticker".to_string();
    }
    String::new()
}

pub async fn extract_filter_text(msg: &Message) -> Option<String> {
    if msg.caption().is_some() {
        return msg.caption().map(|s| s.to_string());
    } else if msg.text().is_some() {
        return msg.text().map(|s| s.to_string());
    } else if msg.sticker().is_some() {
        return msg.sticker().map(|s| s.clone().emoji.unwrap());
    }
    None
}
