use regex::Regex;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{GetChatId, Requester};
use teloxide::types::{
    ChatKind, ChatMemberStatus, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode,
};
use teloxide::utils::command::parse_command;
use teloxide::utils::html::{user_mention, user_mention_or_link};

use crate::database::db_utils::{
    get_softwarn, get_warn_count, get_warn_limit, insert_warn, reset_warn, rm_single_warn,
    set_softwarn, set_warn_limit,
};
use crate::database::{Warn, WarnKind, Warnlimit};
use crate::util::{
    extract_text_id_from_reply, get_bot_id, is_group, is_user_admin, user_should_be_admin,
    user_should_restrict, WarnMode,
};
use crate::{get_mdb, Ctx, Cxt, TgErr};
pub async fn warn(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let (user_id, text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targeted").await?;
        return Ok(());
    }
    let reason = text.unwrap_or_else(String::new);
    warn_user(cx, user_id.unwrap(), reason).await?;
    Ok(())
}

pub async fn warn_user(cx: &Cxt, id: i64, reason: String) -> TgErr<()> {
    let bot_id = get_bot_id(&cx).await;
    let db = get_mdb().await;
    if id == bot_id {
        cx.reply_to("I am not gonna warn myself fella! Try using your brain next time!")
            .await?;
        return Ok(());
    }

    if is_user_admin(cx, id).await {
        cx.reply_to("I am not gonna warn an admin here!").await?;
        return Ok(());
    }
    let w_count = get_warn_count(&db, cx.chat_id(), id).await?;
    let lim = get_warn_limit(&db, cx.chat_id()).await?;
    let mode = get_softwarn(&db, cx.chat_id()).await?;
    let warn = &Warn {
        chat_id: cx.chat_id(),
        user_id: id,
        reason: reason.clone(),
        count: (w_count + 1) as u64,
    };
    if let Ok(mem) = cx.requester.get_chat_member(cx.chat_id(), id).await {
        if (w_count + 1) >= lim {
            cx.requester.kick_chat_member(cx.chat_id(), id).await?;
            if mode {
                cx.requester.unban_chat_member(cx.chat_id(), id).await?;
                cx.reply_to(format!(
                    "That's it get out ({}\\{}) warns, User has been kicked!",
                    &w_count + 1,
                    &lim
                ))
                .await?;
            } else {
                cx.reply_to(format!(
                    "That's it get out ({}\\{}) warns, User has been banned!",
                    &w_count + 1,
                    &lim
                ))
                .await?;
            }
            reset_warn(&db, cx.chat_id(), id).await?;
            return Ok(());
        }
        let rm_warn_data = format!("rm_warn({},{})", cx.chat_id(), id);
        let warn_button = InlineKeyboardButton::callback("Remove Warn".to_string(), rm_warn_data);
        insert_warn(&db, warn).await?;
        cx.reply_to(format!(
            "Warned {}({}\\{})\nReason:{}",
            user_mention_or_link(&mem.user),
            &w_count + 1,
            &lim,
            &reason
        ))
        .reply_markup(InlineKeyboardMarkup::default().append_row(vec![warn_button]))
        .await?;
    } else {
        cx.reply_to("Can't get info about the user").await?;
    }
    Ok(())
}
pub async fn handle_unwarn_button(cx: &Ctx) -> TgErr<()> {
    let db = get_mdb().await;
    let data = &cx.update.data;
    if let Some(d) = data {
        let re = Regex::new(r#"rm_warn\((.+?),(.+?)\)"#).unwrap();
        let caps = re.captures(&d).unwrap();
        let chat_id = caps
            .get(1)
            .map_or(0_i64, |s| s.as_str().parse::<i64>().unwrap());
        let user_id = cx.update.from.id;
        let warned_user = caps
            .get(2)
            .map_or(0_i64, |s| s.as_str().parse::<i64>().unwrap());
        let chatmem = cx.requester.get_chat_member(chat_id, user_id).await?;
        let count = get_warn_count(&db, chat_id, warned_user).await?;
        match chatmem.status() {
            ChatMemberStatus::Administrator | ChatMemberStatus::Creator => {
                if count == 0 || count.is_negative() {
                    cx.requester
                        .edit_message_text(
                            chat_id,
                            cx.update.message.clone().unwrap().id,
                            "Warn is alredy removed",
                        )
                        .await?;
                    return Ok(());
                }
                rm_single_warn(&db, chat_id, warned_user).await?;
                cx.requester
                    .edit_message_text(
                        chat_id,
                        cx.update.message.clone().unwrap().id,
                        format!("Warn Removed by {}", user_mention_or_link(&cx.update.from)),
                    )
                    .await?;
            }
            _ => {
                return Ok(());
            }
        }
    }
    Ok(())
}
pub async fn warn_limit(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), "consts::BOT_NAME").unwrap();
    if args.is_empty() {
        cx.reply_to("Send proper warn limit").await?;
        return Ok(());
    }
    let lim = match args[0].parse::<u64>() {
        Ok(val) => val,
        Err(_) => {
            cx.reply_to("Send a proper warn limit you idiot!").await?;
            return Ok(());
        }
    };
    let wl = &Warnlimit {
        chat_id: cx.chat_id(),
        limit: lim,
    };
    set_warn_limit(&db, wl).await?;
    cx.reply_to(format!("Warn limit set to {}", lim)).await?;
    Ok(())
}

pub async fn warnmode(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id),
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), "consts::BOT_NAME").unwrap();
    if args.is_empty() {
        cx.reply_to("Mention any option! Available one's are (soft/smooth),(hard/strong)")
            .await?;
        return Ok(());
    }
    let mode = args[0].to_lowercase().parse::<WarnMode>().unwrap();
    match mode {
        WarnMode::Soft => {
            let wk = &WarnKind {
                chat_id: cx.chat_id(),
                softwarn: true,
            };
            set_softwarn(&db, wk).await?;
            cx.reply_to("Warnmode is set to soft. Now I will just kick the chat members when their warn exceeds the limit").await?;
        }
        WarnMode::Hard => {
            let wk = &WarnKind {
                chat_id: cx.chat_id(),
                softwarn: false,
            };
            set_softwarn(&db, wk).await?;
            cx.reply_to("Warnmode is set to hard. I will ban the chat members when their warn exceeds the limit").await?;
        }
        WarnMode::Error => {
            cx.reply_to("Invalid warnmode available one's are (soft/strong),(strong/hard)")
                .await?;
        }
    }
    Ok(())
}

pub async fn reset_warns(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id),
    )?;
    let db = get_mdb().await;
    let (user_id, _) = extract_text_id_from_reply(cx).await;
    let count = get_warn_count(&db, cx.chat_id(), user_id.unwrap()).await?;
    if let Ok(member) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if count > 0 {
            reset_warn(&db, cx.chat_id(), user_id.unwrap()).await?;
            cx.reply_to(format!(
                "Warns has been reset for the user {}",
                user_mention_or_link(&member.user)
            ))
            .await?;
        } else {
            cx.reply_to("User was not warned even once").await?;
        }
    } else {
        cx.reply_to("This user is not in the chat I can't unwarn him ")
            .await?;
    }
    Ok(())
}

pub async fn warns(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id),
    )?;
    let db = get_mdb().await;
    let limit = get_warn_limit(&db, cx.chat_id()).await?;
    let (user_id, _) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        if let Ok(chat) = cx.requester.get_chat(cx.chat_id()).await {
            if let ChatKind::Public(c) = chat.kind {
                cx.reply_to(format!("The chat {} has warn limit of {} when the warns exceed the limit the user will be banned from the group",c.title.clone().unwrap(),limit)).await?;
                return Ok(());
            }
        }
    }
    if let Ok(chatmem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        let count = get_warn_count(&db, cx.chat_id(), user_id.unwrap()).await?;
        if count <= 0 {
            cx.reply_to("This user got no warnings").await?;
            return Ok(());
        }
        cx.reply_to(format!(
            "User {} got {}/{} warnings",
            user_mention(user_id.unwrap(), &chatmem.user.full_name()),
            count,
            limit
        ))
        .parse_mode(ParseMode::Html)
        .await?;
    }
    Ok(())
}
