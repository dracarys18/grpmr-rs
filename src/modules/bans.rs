use teloxide::{
    payloads::{KickChatMemberSetters, SendMessageSetters},
    prelude::{GetChatId, Requester},
    types::{ChatMemberStatus, ParseMode},
    utils::html::{self, user_mention_or_link},
};

use crate::{
    database::db_utils::get_log_channel,
    get_mdb,
    modules::send_log,
    util::{
        check_command_disabled, extract_text_id_from_reply, get_bot_id, get_chat_title, get_time,
        is_group, sudo_or_owner_filter, user_should_restrict, TimeUnit,
    },
    Cxt, TgErr, OWNER_ID, SUDO_USERS,
};

pub async fn ban(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let db = get_mdb().await;
    let bot_id = get_bot_id(&cx).await;
    let (user_id, text) = extract_text_id_from_reply(cx).await;
    let reason = text.unwrap_or_else(|| String::from("None"));
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
        if let ChatMemberStatus::Kicked = mem.status() {
            cx.reply_to("This user is already banned here!").await?;
            return Ok(());
        }
        if let ChatMemberStatus::Creator = mem.status() {
            cx.reply_to("I am not gonna ban an Admin Here!").await?;
            return Ok(());
        }
        if let ChatMemberStatus::Administrator = mem.status() {
            if !mem.can_be_edited() {
                cx.reply_to("I am not gonna ban an Admin Here!").await?;
                return Ok(());
            }
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
    let ban_text = format!(
        "<b>Banned</b>\n<b>User:</b>{}\n\n<i>Reason:</i> {}",
        user_mention_or_link(&user),
        reason
    );
    cx.requester
        .kick_chat_member(cx.chat_id(), user_id.unwrap())
        .await?;
    cx.reply_to(ban_text).parse_mode(ParseMode::Html).await?;

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
            "Chat Title: {}\n#BANNED\nAdmin: {}\nUser: {}",
            html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
            html::user_mention(admin.id, &admin.full_name()),
            html::user_mention(user_id.unwrap(), &user.full_name())
        );
        send_log(cx, &logm, l).await?;
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
    let bot_id = get_bot_id(cx).await;
    let db = get_mdb().await;
    if user_id.is_none() {
        cx.reply_to("No user was targetted").await?;
        return Ok(());
    }

    if text.is_none() {
        cx.reply_to("Mention muting time in s,m,h,d").await?;
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
        if matches!(mem.status(), ChatMemberStatus::Creator) {
            cx.reply_to("I am not gonna ban an admin here").await?;
            return Ok(());
        }

        if matches!(mem.status(), ChatMemberStatus::Administrator) && !mem.can_be_edited() {
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
        cx.reply_to(format!("<b>Banned for <i>{}</i></b> ", u.as_ref().unwrap()))
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
                "Chat title: {}\n#TEMP_BANNED\nAdmin: {}\nUser: {}\n For: {}\n",
                html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
                html::user_mention(admin.id, &admin.full_name()),
                html::user_mention(user_id.unwrap(), &mem.user.full_name()),
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

pub async fn unban(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
        user_should_restrict(cx, get_bot_id(cx).await),         //Bot Should have restrict rights
        user_should_restrict(cx, cx.update.from().unwrap().id), //User should have restrict rights
    )?;
    let db = get_mdb().await;
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
            "Chat title: {}\n#UNBANNED\nAdmin: {}\nUser: {}",
            html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
            html::user_mention(admin.id, &admin.full_name()),
            html::user_mention(user_id.unwrap(), &user.full_name())
        );
        send_log(cx, &logm, l).await?;
    }
    Ok(())
}

pub async fn kick(cx: &Cxt) -> TgErr<()> {
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
        if matches!(
            mem.status(),
            ChatMemberStatus::Kicked | ChatMemberStatus::Left
        ) {
            cx.reply_to("This user is already gone mate!").await?;
            return Ok(());
        }
        if matches!(mem.status(), ChatMemberStatus::Creator) {
            cx.reply_to("I am not gonna kick an Admin Here!").await?;
            return Ok(());
        }
        if matches!(mem.status(), ChatMemberStatus::Administrator) && !mem.can_be_edited() {
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
    let reason = text.unwrap_or_else(|| String::from("None"));
    let kick_text = format!(
        "<b>Kicked</b>\n<b>User:</b>{}\n\n<i>Reason:</i> {}",
        user_mention_or_link(&user),
        reason
    );
    cx.requester
        .kick_chat_member(cx.chat_id(), user_id.unwrap())
        .await?;
    cx.requester
        .unban_chat_member(cx.chat_id(), user_id.unwrap())
        .await?;
    cx.reply_to(kick_text).parse_mode(ParseMode::Html).await?;
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
            "Chat title: {}\n#KICKED\nAdmin: {}\nUser: {}",
            html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
            html::user_mention(admin.id, &admin.full_name()),
            html::user_mention(user_id.unwrap(), &user.full_name())
        );
        send_log(cx, &logm, l).await?;
    }
    Ok(())
}
pub async fn kickme(cx: &Cxt, cmd: &str) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_restrict(cx, get_bot_id(cx).await),
        check_command_disabled(cx, String::from(cmd))
    )?;
    let db = get_mdb().await;
    if let Some(user) = cx.update.from() {
        let user_id = user.id;
        if user_id == *OWNER_ID || (*SUDO_USERS).contains(&user_id) {
            cx.reply_to("You are my owner or one of my sudo users mate I can't kick you")
                .await?;
            return Ok(());
        }
        if let Ok(mem) = cx.requester.get_chat_member(cx.chat_id(), user_id).await {
            if matches!(mem.status(), ChatMemberStatus::Creator) {
                cx.reply_to("I am not gonna kick an Admin Here!").await?;
                return Ok(());
            }
            if matches!(mem.status(), ChatMemberStatus::Administrator) && !mem.can_be_edited() {
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
        if let Some(l) = get_log_channel(&db, cx.chat_id()).await? {
            let user = cx
                .requester
                .get_chat_member(cx.chat_id(), user_id)
                .await?
                .user;
            let logm = format!(
                "Chat id: {}\n#KICKME\nUser: {}",
                html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
                html::user_mention(user_id, &user.full_name())
            );
            send_log(cx, &logm, l).await?;
        }
    } else {
        cx.reply_to("Can't get the info about the user").await?;
    }
    Ok(())
}
