use crate::util::{
    can_pin_messages, can_send_text, extract_text_id_from_reply, get_bot_id, is_group,
    is_user_restricted, user_should_restrict, PinMode,
};
use crate::{Cxt, TgErr};
use std::str::FromStr;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberKind, ChatMemberStatus, ChatPermissions, ParseMode};
use teloxide::utils::command::parse_command;
use teloxide::utils::html::user_mention_or_link;

pub async fn ban(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let bot_id = get_bot_id(&cx).await;
    let (user_id, _text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targeted").await?;
        return Ok(());
    }
    if user_id.unwrap() == bot_id {
        cx.reply_to("I am not gonna ban myself fella! Try using your brain next time!")
            .await?;
        return Ok(());
    }

    if let Ok(mem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if let ChatMemberKind::Kicked(_) = mem.kind {
            cx.reply_to("This user is already banned here!").await?;
            return Ok(());
        }
        if let ChatMemberKind::Administrator(_) | ChatMemberKind::Creator(_) = mem.kind {
            cx.reply_to("I am not gonna ban an Admin Here!").await?;
            return Ok(());
        }
    } else {
        cx.reply_to("I can't seem to get info for this user")
            .await?;
        return Ok(());
    };
    let user = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
        .unwrap()
        .user;
    let ban_text = format!("<b>Banned</b>\n<b>User:</b>{}", user_mention_or_link(&user));
    cx.requester
        .kick_chat_member(cx.chat_id(), user_id.unwrap())
        .await?;
    cx.reply_to(ban_text).parse_mode(ParseMode::Html).await?;
    Ok(())
}

pub async fn unban(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let (user_id, _text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targeted").await?;
        return Ok(());
    }

    if let Ok(mem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if !matches!(mem.status(), ChatMemberStatus::Kicked) {
            cx.reply_to("This user is already unbanned!").await?;
            return Ok(());
        }
    } else {
        cx.reply_to("I can't seem to get the info of the user")
            .await?;
        return Ok(());
    }

    cx.requester
        .unban_chat_member(cx.chat_id(), user_id.unwrap())
        .await?;
    cx.reply_to("<b>Unbanned!</b>")
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
pub async fn mute(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let bot_id = get_bot_id(&cx).await;
    let (user_id, _text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targeted").await?;
        return Ok(());
    }
    if user_id.unwrap() == bot_id {
        cx.reply_to("I am not gonna mute myself fella! Try using your brain next time!")
            .await?;
        return Ok(());
    }
    if let Ok(mem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if let ChatMemberKind::Administrator(_) | ChatMemberKind::Creator(_) = mem.kind {
            cx.reply_to("I am not gonna mute an Admin Here!").await?;
            return Ok(());
        }
    } else {
        cx.reply_to("I can't seem to get info for this user")
            .await?;
        return Ok(());
    }
    let user = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
        .unwrap()
        .user;
    if can_send_text(cx, user_id.unwrap()).await? {
        cx.reply_to("User is already restricted").await?;
        return Ok(());
    }
    let mute_text = format!("<b>Muted</b>\n<b>User:</b>{}", user_mention_or_link(&user));
    cx.requester
        .restrict_chat_member(cx.chat_id(), user_id.unwrap(), ChatPermissions::default())
        .await?;
    cx.reply_to(mute_text).parse_mode(ParseMode::Html).await?;
    Ok(())
}
pub async fn unmute(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let perm: ChatPermissions = ChatPermissions::new()
        .can_send_messages(true)
        .can_send_media_messages(true)
        .can_send_other_messages(true)
        .can_send_polls(true)
        .can_add_web_page_previews(true);
    let (user_id, _text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targeted").await?;
        return Ok(());
    }
    let member = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await?;

    if matches!(
        member.status(),
        ChatMemberStatus::Kicked | ChatMemberStatus::Left
    ) {
        cx.reply_to("This user already banned/left from the group")
            .await?;
        return Ok(());
    }
    if !is_user_restricted(cx, user_id.unwrap()).await? {
        cx.reply_to("This user can already talk!").await?;
        return Ok(());
    }
    cx.requester
        .restrict_chat_member(cx.chat_id(), user_id.unwrap(), perm)
        .await?;
    cx.reply_to("<b>Unmuted</b>")
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
pub async fn kick(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let bot_id = get_bot_id(&cx).await;
    let (user_id, _text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targeted").await?;
        return Ok(());
    }
    if user_id.unwrap() == bot_id {
        cx.reply_to("I am not gonna kick myself fella! Try using your brain next time!")
            .await?;
        return Ok(());
    }

    if let Ok(mem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if let ChatMemberStatus::Kicked | ChatMemberStatus::Left = mem.status() {
            cx.reply_to("This user is already gone mate!").await?;
            return Ok(());
        }
        if let ChatMemberKind::Administrator(_) | ChatMemberKind::Creator(_) = mem.kind {
            cx.reply_to("I am not gonna kick an Admin Here!").await?;
            return Ok(());
        }
    } else {
        cx.reply_to("I can't seem to get info for this user")
            .await?;
        return Ok(());
    };
    let user = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
        .unwrap()
        .user;
    let kick_text = format!("<b>Kicked</b>\n<b>User:</b>{}", user_mention_or_link(&user));
    cx.requester
        .kick_chat_member(cx.chat_id(), user_id.unwrap())
        .await?;
    cx.requester
        .unban_chat_member(cx.chat_id(), user_id.unwrap())
        .await?;
    cx.reply_to(kick_text).parse_mode(ParseMode::Html).await?;
    Ok(())
}
pub async fn kickme(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(is_group(cx), user_should_restrict(cx, get_bot_id(cx).await))?;
    if let Some(user) = cx.update.from() {
        let user_id = user.id;
        if let Ok(mem) = cx.requester.get_chat_member(cx.chat_id(), user_id).await {
            if let ChatMemberKind::Administrator(_) | ChatMemberKind::Creator(_) = mem.kind {
                cx.reply_to("I am not gonna kick an Admin Here!").await?;
                return Ok(());
            }
        } else {
            cx.reply_to("Can't kick the user").await?;
            return Ok(());
        }
        let kickme_text = format!("<b>Piss off {}</b>", user_mention_or_link(user));
        cx.requester.kick_chat_member(cx.chat_id(), user_id).await?;
        cx.requester
            .unban_chat_member(cx.chat_id(), user_id)
            .await?;
        cx.reply_to(kickme_text).await?;
    } else {
        cx.reply_to("Can't get the info about the user").await?;
        return Ok(());
    }
    return Ok(());
}

pub async fn pin(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        can_pin_messages(cx, get_bot_id(cx).await),
        can_pin_messages(cx, cx.update.from().unwrap().id)
    )?;
    let (_, args) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
    if let Some(mes) = cx.update.reply_to_message() {
        if !args.is_empty() {
            let pinmode = PinMode::from_str(&args[0].to_lowercase()).unwrap();
            match pinmode {
                PinMode::Loud => {
                    cx.requester
                        .pin_chat_message(cx.chat_id(), mes.id)
                        .disable_notification(false)
                        .await?;
                    cx.reply_to("Pinned Loudly").await?;
                }
                PinMode::Silent => {
                    cx.requester
                        .pin_chat_message(cx.chat_id(), mes.id)
                        .disable_notification(true)
                        .await?;
                    cx.reply_to("Pinned Silently").await?;
                }
                PinMode::Error => {
                    cx.reply_to("Invalid PinMode! Available pinmodes are loud,hard,violent,silent")
                        .await?;
                }
            }
        } else {
            cx.requester
                .pin_chat_message(cx.chat_id(), mes.id)
                .disable_notification(false)
                .await?;
            cx.reply_to("Pinned").await?;
        }
    } else {
        cx.reply_to("Reply to some message to pin").await?;
    }
    Ok(())
}

pub async fn unpin(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        can_pin_messages(cx, get_bot_id(cx).await),
        can_pin_messages(cx, cx.update.from().unwrap().id),
    )?;
    if let Some(mes) = cx.update.reply_to_message() {
        match cx
            .requester
            .unpin_chat_message(cx.chat_id())
            .message_id(mes.id as i32)
            .await
        {
            Ok(_) => {
                cx.reply_to("Unpinned the mentioned message").await?;
            }
            Err(_) => {
                cx.reply_to("The mentioned message was never pinned")
                    .await?;
            }
        }
    } else {
        match cx.requester.unpin_all_chat_messages(cx.chat_id()).await {
            Ok(_) => {
                cx.reply_to("Unpinned all chat messages").await?;
            }
            Err(_) => {
                cx.reply_to("What are you trying to unpin").await?;
            }
        }
    }
    Ok(())
}
