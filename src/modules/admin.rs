use crate::util::{
    can_pin_messages, can_promote_members, check_command_disabled, consts,
    extract_text_id_from_reply, get_bot_id, is_group, user_should_be_admin, PinMode,
};
use crate::{Cxt, TgErr, OWNER_ID, SUDO_USERS};
use std::str::FromStr;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{ChatKind, ChatMemberStatus, ParseMode};
use teloxide::utils::command::parse_command;
use teloxide::utils::html::{user_mention, user_mention_or_link};

pub async fn pin(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        can_pin_messages(cx, get_bot_id(cx).await),
        can_pin_messages(cx, cx.update.from().unwrap().id)
    )?;
    let (_, args) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();
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

pub async fn promote(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        can_promote_members(cx, get_bot_id(cx).await),
        can_promote_members(cx, cx.update.from().unwrap().id)
    )?;
    let (user_id, text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("Mention someone to promote").await?;
        return Ok(());
    }
    if let Ok(chatmem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if matches!(chatmem.status(), ChatMemberStatus::Creator) {
            cx.reply_to("Mate the user is the creator of the group")
                .await?;
            return Ok(());
        }
        let promote_text;
        if matches!(chatmem.status(), ChatMemberStatus::Administrator) {
            if !chatmem.kind.can_be_edited() {
                cx.reply_to("I dont have enough rights to update the user's permissons!")
                    .await?;
                return Ok(());
            }
            promote_text = format!(
                "Admin Permissions has been updated for\n <b>User:</b>{}",
                user_mention_or_link(&chatmem.user)
            );
        } else {
            promote_text = format!(
                "Promoted\n<b>User:</b>{}",
                user_mention_or_link(&chatmem.user)
            );
        }
        cx.requester
            .promote_chat_member(cx.chat_id(), user_id.unwrap())
            .can_manage_chat(true)
            .can_change_info(true)
            .can_delete_messages(true)
            .can_invite_users(true)
            .can_restrict_members(true)
            .can_pin_messages(true)
            .await?;
        if text.is_some() {
            cx.requester
                .set_chat_administrator_custom_title(cx.chat_id(), user_id.unwrap(), text.unwrap())
                .await?;
        }
        cx.reply_to(promote_text)
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        cx.reply_to("Who are you trying to promote? He is not even in the group")
            .await?;
    }
    Ok(())
}

pub async fn demote(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        can_promote_members(cx, get_bot_id(cx).await),
        can_promote_members(cx, cx.update.from().unwrap().id)
    )?;
    let (user_id, _) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("Mention a user to demote").await?;
        return Ok(());
    }

    if user_id.unwrap() == *OWNER_ID || (*SUDO_USERS).contains(&user_id.unwrap()) {
        cx.reply_to("I can't kick the people who created me! I got loyalty")
            .await?;
        return Ok(());
    }
    if let Ok(chatmem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if matches!(chatmem.status(), ChatMemberStatus::Creator) {
            cx.reply_to("This user is the Creator of the group, How can I possibly demote them")
                .await?;
            return Ok(());
        }

        if !matches!(chatmem.status(), ChatMemberStatus::Administrator) {
            cx.reply_to("The user has to admin in the first place to demote")
                .await?;
            return Ok(());
        }

        if chatmem.kind.can_be_edited() {
            cx.requester
                .promote_chat_member(cx.chat_id(), user_id.unwrap())
                .can_manage_chat(false)
                .can_change_info(false)
                .can_delete_messages(false)
                .can_manage_voice_chats(false)
                .can_invite_users(false)
                .can_restrict_members(false)
                .can_pin_messages(false)
                .can_promote_members(false)
                .await?;
            cx.reply_to(format!(
                "Demoted Successfully\n<b>User:</b>{}",
                user_mention_or_link(&chatmem.user)
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        } else {
            cx.reply_to("I don't seem to have enough rights to demote this user")
                .await?;
            return Ok(());
        }
    } else {
        cx.reply_to("Who are you trying demote?").await?;
    }
    Ok(())
}

pub async fn invitelink(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id)
    )?;
    let chat = &cx.update.chat;
    match &chat.kind {
        ChatKind::Public(c) => {
            if c.invite_link.is_some() {
                cx.reply_to(format!(
                    "<b>Here's the invite link of the chat</b>\n{}",
                    c.invite_link.as_ref().unwrap()
                ))
                .parse_mode(ParseMode::Html)
                .await?;
            } else if let Ok(inv) = cx.requester.export_chat_invite_link(cx.chat_id()).await {
                cx.reply_to(format!(
                    "<b>The invitelink was empty so I have created one for this chat</b>\n{}",
                    inv
                ))
                .parse_mode(ParseMode::Html)
                .await?;
            } else {
                cx.reply_to("I don't have enough rights to access the invite link")
                    .await?;
            }
        }
        ChatKind::Private(_) => {
            cx.reply_to("I can only create invite links for chats or channels")
                .await?;
        }
    }
    Ok(())
}

pub async fn adminlist(cx: &Cxt, cmd: &str) -> TgErr<()> {
    tokio::try_join!(is_group(cx), check_command_disabled(cx, String::from(cmd)))?;
    let chatmem = cx.requester.get_chat_administrators(cx.chat_id()).await?;
    let adminlist = chatmem
        .iter()
        .map(|mem| {
            format!(
                "- {}",
                user_mention(mem.user.id as i32, &mem.user.full_name())
            )
        })
        .collect::<Vec<String>>();
    cx.reply_to(format!(
        "<b>Admin's in this group:</b>\n{}",
        adminlist.join("\n")
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
