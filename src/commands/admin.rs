use crate::util::{
    can_pin_messages, can_promote_members, can_send_text, extract_text_id_from_reply, get_bot_id,
    get_time, is_group, is_user_restricted, sudo_or_owner_filter, user_should_be_admin,
    user_should_restrict, PinMode, TimeUnit,
};
use crate::{Cxt, TgErr, OWNER_ID, SUDO_USERS};
use std::str::FromStr;
use teloxide::prelude::*;
use teloxide::types::{ChatKind, ChatMemberKind, ChatMemberStatus, ChatPermissions, ParseMode};
use teloxide::utils::command::parse_command;
use teloxide::utils::html::{user_mention, user_mention_or_link};

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

    if user_id.unwrap() == *OWNER_ID || (*SUDO_USERS).contains(&user_id.unwrap()) {
        cx.reply_to("I am not gonna ban my owner or my sudo users")
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

pub async fn temp_mute(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let (user_id, text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targetted").await?;
        return Ok(());
    }

    if text.is_none() {
        cx.reply_to("Mention muting time in s,m,h,d").await?;
        return Ok(());
    }

    if let Ok(mem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if matches!(
            mem.status(),
            ChatMemberStatus::Administrator | ChatMemberStatus::Creator
        ) {
            cx.reply_to("I am not gonna mute an admin here").await?;
            return Ok(());
        }

        if matches!(
            mem.status(),
            ChatMemberStatus::Kicked | ChatMemberStatus::Left
        ) {
            cx.reply_to(
                "This user is either left of banned from here there's no point of muting him",
            )
            .await?;
            return Ok(());
        }
        if sudo_or_owner_filter(user_id.unwrap()).await.is_ok() {
            cx.reply_to("This user is either my owner or my sudo user I am not gonna mute him")
                .await?;
            return Ok(());
        }

        if user_id.unwrap() == get_bot_id(&cx).await {
            cx.reply_to("I am not gonna mute myself you idiot!").await?;
            return Ok(());
        }
        let u = text.unwrap().parse::<TimeUnit>();
        if u.is_err() {
            cx.reply_to("Please specify with proper unit: s,m,h,d")
                .await?;
            return Ok(());
        }
        let t = get_time(u.as_ref().unwrap());
        cx.requester
            .restrict_chat_member(cx.chat_id(), user_id.unwrap(), ChatPermissions::default())
            .until_date(cx.update.date as u64 + t)
            .await?;
        cx.reply_to(format!("<b>Muted for <i>{}</i></b> ", u.unwrap()))
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        cx.reply_to("Can't get this user maybe he's not in the group or he deleted his account")
            .await?;
    }

    Ok(())
}

pub async fn temp_ban(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let (user_id, text) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targetted").await?;
        return Ok(());
    }

    if text.is_none() {
        cx.reply_to("Mention muting time in s,m,h,d").await?;
        return Ok(());
    }

    if let Ok(mem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if matches!(
            mem.status(),
            ChatMemberStatus::Administrator | ChatMemberStatus::Creator
        ) {
            cx.reply_to("I am not gonna ban an admin here").await?;
            return Ok(());
        }

        if matches!(mem.status(), ChatMemberStatus::Kicked) {
            cx.reply_to("This user is already banned").await?;
            return Ok(());
        }

        if sudo_or_owner_filter(user_id.unwrap()).await.is_ok() {
            cx.reply_to("This user is either my owner or my sudo user I am not gonna ban him")
                .await?;
            return Ok(());
        }

        if user_id.unwrap() == get_bot_id(&cx).await {
            cx.reply_to("I am not gonna ban myself you idiot!").await?;
            return Ok(());
        }
        let u = text.unwrap().parse::<TimeUnit>();
        if u.is_err() {
            cx.reply_to("Please specify with proper unit: s,m,h,d")
                .await?;
            return Ok(());
        }
        let t = get_time(u.as_ref().unwrap());
        cx.requester
            .kick_chat_member(cx.chat_id(), user_id.unwrap())
            .until_date(cx.update.date as u64 + t)
            .await?;
        cx.reply_to(format!("<b>Banned for <i>{}</i></b> ", u.unwrap()))
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        cx.reply_to("Can't get this user maybe he's not in the group or he deleted his account")
            .await?;
    }

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

    if user_id.unwrap() == *OWNER_ID || (*SUDO_USERS).contains(&user_id.unwrap()) {
        cx.reply_to("I am not gonna mute my owner or one of my sudo users")
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

    if user_id.unwrap() == *OWNER_ID || (*SUDO_USERS).contains(&user_id.unwrap()) {
        cx.reply_to("I am not gonna kick my owner or one of my sudo users")
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
        if user_id == *OWNER_ID || (*SUDO_USERS).contains(&user_id) {
            cx.reply_to("You are my owner or one of my sudo users mate I can't kick you")
                .await?;
            return Ok(());
        }
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
            } else {
                if let Ok(inv) = cx.requester.export_chat_invite_link(cx.chat_id()).await {
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
        }
        ChatKind::Private(_) => {
            cx.reply_to("I can only create invite links for chats or channels")
                .await?;
        }
    }
    Ok(())
}

pub async fn adminlist(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(is_group(cx))?;
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
