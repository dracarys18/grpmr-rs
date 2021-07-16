use crate::database::db_utils::{get_report_setting, set_report_setting};
use crate::database::Reporting;
use crate::util::{
    can_pin_messages, can_promote_members, check_command_disabled, consts,
    extract_text_id_from_reply, get_bot_id, get_chat_title, is_group, is_user_admin,
    user_should_be_admin, PinMode, ReportStatus,
};
use crate::{get_mdb, Cxt, TgErr, OWNER_ID, SUDO_USERS};
use std::str::FromStr;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{ChatKind, ParseMode};
use teloxide::utils::command::parse_command;
use teloxide::utils::html::{self, user_mention, user_mention_or_link};

pub async fn pin(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        can_pin_messages(cx, get_bot_id(cx).await),
        can_pin_messages(cx, cx.update.from().unwrap().id)
    )?;
    let (_, args) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();
    if let Some(mes) = cx.update.reply_to_message() {
        let pinmsg = html::link(mes.url().unwrap().as_str(), "this message");
        if !args.is_empty() {
            let pinmode = PinMode::from_str(&args[0].to_lowercase()).unwrap();
            match pinmode {
                PinMode::Loud => {
                    cx.requester
                        .pin_chat_message(cx.chat_id(), mes.id)
                        .disable_notification(false)
                        .await?;
                    cx.reply_to(format!("Pinned {} Loudly", &pinmsg))
                        .disable_web_page_preview(true)
                        .parse_mode(ParseMode::Html)
                        .await?;
                }
                PinMode::Silent => {
                    cx.requester
                        .pin_chat_message(cx.chat_id(), mes.id)
                        .disable_notification(true)
                        .await?;
                    cx.reply_to(format!("Pinned {} Silently", &pinmsg))
                        .disable_web_page_preview(true)
                        .parse_mode(ParseMode::Html)
                        .await?;
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
            cx.reply_to(format!("Pinned {}", &pinmsg))
                .disable_web_page_preview(true)
                .parse_mode(ParseMode::Html)
                .await?;
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
                cx.reply_to(format!(
                    "Unpinned {}",
                    html::link(mes.url().unwrap().as_str(), "this message")
                ))
                .disable_web_page_preview(true)
                .parse_mode(ParseMode::Html)
                .await?;
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
    let botmem = cx
        .requester
        .get_chat_member(cx.chat_id(), get_bot_id(cx).await)
        .await?;
    if user_id.is_none() {
        cx.reply_to("Mention someone to promote").await?;
        return Ok(());
    }
    if let Ok(chatmem) = cx
        .requester
        .get_chat_member(cx.chat_id(), user_id.unwrap())
        .await
    {
        if chatmem.is_owner() {
            cx.reply_to("Mate the user is the creator of the group")
                .await?;
            return Ok(());
        }
        let promote_text = if chatmem.is_administrator() {
            if !chatmem.kind.can_be_edited() {
                cx.reply_to("I dont have enough rights to update the user's permissons!")
                    .await?;
                return Ok(());
            }
            format!(
                "Admin Permissions has been updated for\n <b>User:</b>{}",
                user_mention_or_link(&chatmem.user)
            )
        } else {
            format!(
                "Promoted\n<b>User:</b>{}",
                user_mention_or_link(&chatmem.user)
            )
        };
        cx.requester
            .promote_chat_member(cx.chat_id(), user_id.unwrap())
            .can_manage_chat(botmem.can_manage_chat())
            .can_change_info(botmem.can_change_info())
            .can_delete_messages(botmem.can_delete_messages())
            .can_invite_users(botmem.can_invite_users())
            .can_restrict_members(botmem.can_restrict_members())
            .can_pin_messages(botmem.can_pin_messages())
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
        if chatmem.is_owner() {
            cx.reply_to("This user is the Creator of the group, How can I possibly demote them")
                .await?;
            return Ok(());
        }

        if !chatmem.is_administrator() {
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
        .map(|mem| format!("- {}", user_mention(mem.user.id, &mem.user.full_name())))
        .collect::<Vec<String>>();
    cx.reply_to(format!(
        "<b>Admin's in this group:</b>\n{}",
        adminlist.join("\n")
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn report_set(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(is_group(cx))?;
    let db = get_mdb().await;
    let (_, arg) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();
    if arg.is_empty() {
        cx.reply_to(format!(
            "Invalid option!\nUsage: {}",
            html::code_inline("/reports on/off/yes/no")
        ))
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }
    let option = arg[0].to_lowercase().parse::<ReportStatus>().unwrap();
    match option {
        ReportStatus::On => {
            let r = Reporting {
                chat_id: cx.chat_id(),
                allowed: true,
            };
            set_report_setting(&db, &r).await?;
            cx.reply_to("Reporting has been turned on for this chat now user's can report any users by sending /report").await?;
        }
        ReportStatus::Off => {
            let r = Reporting {
                chat_id: cx.chat_id(),
                allowed: false,
            };
            set_report_setting(&db, &r).await?;
            cx.reply_to("Reporting has been turned on for this chat now user's can report any users by sending /report").await?;
        }
        ReportStatus::Error => {
            cx.reply_to(format!(
                "Invalid option!\nUsage: {}",
                html::code_inline("/reports on/off")
            ))
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}
pub async fn report(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(is_group(cx))?;
    let db = get_mdb().await;

    // Early return if the reporting is false in a group or someone trying to bluetext spam
    if !get_report_setting(&db, cx.chat_id()).await? || cx.update.reply_to_message().is_none() {
        return Ok(());
    }
    let repo_msg = cx.update.reply_to_message().unwrap();
    let culprit = repo_msg.from().unwrap();

    if culprit.id == get_bot_id(cx).await {
        cx.reply_to("I am not reporting myself you cretin").await?;
        return Ok(());
    }

    //User is reporting himself, spam?
    if culprit.id == cx.update.from().unwrap().id {
        cx.reply_to("Why are you reporting yourself").await?;
        return Ok(());
    }

    //Ignore if user is trying to report an admin
    if is_user_admin(cx, culprit.id).await {
        return Ok(());
    }

    let adminlist = cx.requester.get_chat_administrators(cx.chat_id()).await?;
    let report_msg = format!(
        "Chat Title: {}\nReport Message: {}\nUser: {}\nReported User: {}",
        html::code_inline(get_chat_title(cx, cx.chat_id()).await.unwrap().as_str()),
        html::link(repo_msg.url().unwrap().as_str(), "this message"),
        html::user_mention(culprit.id, &culprit.full_name()),
        html::user_mention(
            cx.update.from().unwrap().id,
            cx.update.from().unwrap().full_name().as_str()
        )
    );
    for ad in adminlist {
        //Can't message a bot
        if ad.user.is_bot {
            continue;
        }
        //Forward the reported message to the admin and don't bother if there's an error
        cx.requester
            .forward_message(ad.user.id, cx.chat_id(), repo_msg.id)
            .await
            .ok();

        //Send the report and don't bother if there's any error
        cx.requester
            .send_message(ad.user.id, &report_msg)
            .parse_mode(ParseMode::Html)
            .disable_web_page_preview(true)
            .await
            .ok();
    }
    cx.reply_to(format!(
        "Reported {} to admins",
        html::user_mention(culprit.id, &culprit.full_name())
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
