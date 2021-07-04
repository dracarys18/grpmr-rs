use teloxide::{
    payloads::{RestrictChatMemberSetters, SendMessageSetters},
    prelude::{GetChatId, Requester},
    types::{ChatMemberStatus, ChatPermissions, ParseMode},
    utils::{
        command::parse_command,
        html::{self, user_mention_or_link},
    },
};

use crate::{
    database::db_utils::get_log_channel,
    get_mdb,
    modules::send_log,
    util::{
        can_send_text, extract_text_id_from_reply, get_bot_id, get_chat_title, get_time, is_group,
        is_user_restricted, sudo_or_owner_filter, user_should_restrict, LockType, TimeUnit,
    },
    Cxt, TgErr, OWNER_ID, SUDO_USERS,
};

pub async fn temp_mute(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let db = get_mdb().await;
    let (user_id, text) = extract_text_id_from_reply(cx).await;
    let bot_id = get_bot_id(cx).await;
    if user_id.is_none() {
        cx.reply_to("No user was targetted").await?;
        return Ok(());
    }

    if text.is_none() {
        cx.reply_to("Mention muting time in s,m,h,d").await?;
        return Ok(());
    }

    if user_id.unwrap() == bot_id {
        cx.reply_to("I am not gonna mute myself fella! Try using your brain next time!")
            .await?;
        return Ok(());
    }

    if user_id.unwrap() == *OWNER_ID || (*SUDO_USERS).contains(&user_id.unwrap()) {
        cx.reply_to("I am not gonna mute my owner or my sudo users")
            .await?;
        return Ok(());
    }

    if let Ok(mem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if matches!(mem.status(), ChatMemberStatus::Creator) {
            cx.reply_to("I am not gonna mute an admin here").await?;
            return Ok(());
        }

        if matches!(mem.status(), ChatMemberStatus::Administrator) && !mem.can_be_edited() {
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
        cx.reply_to(format!("<b>Muted for <i>{}</i></b> ", u.as_ref().unwrap()))
            .parse_mode(ParseMode::Html)
            .await?;
        if let Some(l) = get_log_channel(&db, cx.chat_id()).await? {
            let admin = cx
                .requester
                .get_chat_member(cx.chat_id(), cx.update.from().unwrap().id)
                .await?
                .user;
            let mem = cx
                .requester
                .get_chat_member(cx.chat_id(), user_id.unwrap())
                .await?;
            let logm = format!(
                "Chat title: {}\n#TEMP_MUTED\nAdmin: {}\nUser: {}\n For: {}\n",
                html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
                html::user_mention(admin.id as i32, &admin.full_name()),
                html::user_mention(user_id.unwrap() as i32, &mem.user.full_name()),
                html::code_inline(&u.unwrap().to_string())
            );
            send_log(cx, &logm, l).await?;
        }
    } else {
        cx.reply_to("Can't get this user maybe he's not in the group or he deleted his account")
            .await?;
    }

    Ok(())
}

pub async fn mute(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let db = get_mdb().await;
    let bot_id = get_bot_id(&cx).await;
    let (user_id, text) = extract_text_id_from_reply(cx).await;
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
        if matches!(mem.status(), ChatMemberStatus::Creator) {
            cx.reply_to("I am not gonna mute an Admin Here!").await?;
            return Ok(());
        }
        if matches!(mem.status(), ChatMemberStatus::Administrator) && !mem.can_be_edited() {
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
    let reason = text.unwrap_or_else(|| String::from("None"));
    let mute_text = format!(
        "<b>Muted</b>\n<b>User:</b>{}\n\n<i>Reason:</i> {}",
        user_mention_or_link(&user),
        reason
    );
    cx.requester
        .restrict_chat_member(cx.chat_id(), user_id.unwrap(), ChatPermissions::default())
        .await?;
    cx.reply_to(mute_text).parse_mode(ParseMode::Html).await?;
    if let Some(l) = get_log_channel(&db, cx.chat_id()).await? {
        let admin = cx
            .requester
            .get_chat_member(cx.chat_id(), cx.update.from().unwrap().id)
            .await?
            .user;
        let user = cx
            .requester
            .get_chat_member(cx.chat_id(), user_id.unwrap())
            .await?
            .user;
        let logm = format!(
            "Chat title: {}\n#MUTED\nAdmin: {}\nUser: {}",
            html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
            html::user_mention(admin.id as i32, &admin.full_name()),
            html::user_mention(user_id.unwrap() as i32, &user.full_name())
        );
        send_log(cx, &logm, l).await?;
    }
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

pub async fn lock(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let (_, args) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
    if args.is_empty() {
        cx.reply_to("What should I lock?").await?;
        return Ok(());
    }
    let locktype = args[0].to_lowercase().parse::<LockType>().unwrap();
    if let Ok(chat) = cx.requester.get_chat(cx.chat_id()).await {
        match locktype {
            LockType::Text(_) => {
                cx.requester
                    .set_chat_permissions(cx.chat_id(), ChatPermissions::default())
                    .await?;
                cx.reply_to(format!("{}", LockType::Text("Lock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Other(_) => {
                let perm = chat.permissions().unwrap().can_send_other_messages(false);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Other("Lock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Media(_) => {
                let perm = chat
                    .permissions()
                    .unwrap()
                    .can_send_media_messages(false)
                    .can_send_other_messages(false)
                    .can_add_web_page_previews(false);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Media("Lock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Poll(_) => {
                let perm = chat.permissions().unwrap().can_send_polls(false);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Poll("Lock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Web(_) => {
                let perm = chat.permissions().unwrap().can_add_web_page_previews(false);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Web("Lock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Error(_) => {
                cx.reply_to(format!("{}", locktype)).await?;
            }
        }
    } else {
        cx.reply_to("Can't get info about the chat").await?;
    }
    Ok(())
}

pub async fn unlock(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let (_, args) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
    if args.is_empty() {
        cx.reply_to("What should I unlock?").await?;
        return Ok(());
    }
    let locktype = args[0].to_lowercase().parse::<LockType>().unwrap();
    if let Ok(chat) = cx.requester.get_chat(cx.chat_id()).await {
        match locktype {
            LockType::Text(_) => {
                let perm = chat
                    .permissions()
                    .unwrap()
                    .can_send_messages(true)
                    .can_add_web_page_previews(true)
                    .can_send_media_messages(true)
                    .can_send_other_messages(true)
                    .can_send_polls(true);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Text("Unlock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Other(_) => {
                let perm = chat.permissions().unwrap().can_send_other_messages(true);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Other("Unlock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Media(_) => {
                let perm = chat.permissions().unwrap().can_send_media_messages(true);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Media("Unlock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Poll(_) => {
                let perm = chat.permissions().unwrap().can_send_polls(true);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Poll("Unlock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Web(_) => {
                let perm = chat.permissions().unwrap().can_add_web_page_previews(true);
                cx.requester
                    .set_chat_permissions(cx.chat_id(), perm)
                    .await?;
                cx.reply_to(format!("{}", LockType::Web("Unlock".to_string())))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            LockType::Error(_) => {
                cx.reply_to(format!("{}", locktype)).await?;
            }
        }
    } else {
        cx.reply_to("Can't get info about the chat").await?;
    }
    Ok(())
}

pub async fn locktypes(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(is_group(cx),)?;
    cx.reply_to("Following Locktypes are available: \n<code>all\ntext\nsticker\ngif\nurl\nweb\nmedia\npoll\n</code>").parse_mode(ParseMode::Html).await?;
    Ok(())
}
