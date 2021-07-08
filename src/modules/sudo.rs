use crate::database::{
    db_utils::{gban_user, get_all_chats, is_gbanned, set_gbanstat, ungban_user},
    Gban, GbanStat,
};
use crate::util::{
    extract_text_id_from_reply, get_bot_id, is_group, owner_filter, sudo_or_owner_filter,
    user_should_be_admin, GbanStats,
};
use crate::{get_mdb, Cxt, TgErr, OWNER_ID, SUDO_USERS};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{ChatKind, ParseMode};
use teloxide::utils::command::parse_command;
use teloxide::utils::html;

pub async fn leavechat(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(owner_filter(cx.update.from().unwrap().id),)?;
    let (_, txt) = parse_command(cx.update.text().unwrap(), "consts::BOT_NAME").unwrap();
    let args = txt.get(0);
    if args.is_none() {
        cx.reply_to("Mention a chat id to leave").await?;
        return Ok(());
    }
    let chat_id = args.unwrap().parse::<i64>().unwrap_or(0);
    if let Ok(chat) = cx.requester.get_chat(chat_id).await {
        match chat.kind {
            ChatKind::Public(pu) => {
                cx.requester.leave_chat(chat_id).await?;
                cx.requester
                    .send_message(
                        *OWNER_ID,
                        format!(
                            "I have left <code>{}</code> boss",
                            html::escape(&pu.title.unwrap())
                        ),
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                return Ok(());
            }
            ChatKind::Private(_) => {
                cx.reply_to(
                    "The chat id you provided belongs to some user I can only leave groups",
                )
                .await?;
                return Ok(());
            }
        }
    } else {
        cx.reply_to("Either you gave me a non-valid chat id or I have been kicked from that group")
            .await?;
    }
    Ok(())
}

pub async fn chatlist(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(owner_filter(cx.update.from().unwrap().id))?;
    let db = get_mdb().await;
    let chatlist = get_all_chats(&db).await?;
    let mut chat_string = String::new();
    for c in chatlist {
        if let Ok(chat) = cx.requester.get_chat(c).await {
            if let ChatKind::Public(pu) = chat.kind {
                let s = format!("- <code>{} : {}</code>\n", pu.title.unwrap(), &c);
                chat_string.push_str(&s);
            }
        }
    }
    cx.reply_to(format!(
        "The chat id's of the chats I am in \n <code>{}</code> ",
        chat_string
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn gban(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(sudo_or_owner_filter(cx.update.from().unwrap().id))?;
    let (user_id, text) = extract_text_id_from_reply(cx).await;
    let db = get_mdb().await;
    if user_id.is_none() {
        cx.reply_to("Specify someone to gban").await?;
        return Ok(());
    }
    let reason = text.unwrap_or_else(String::new);
    let gb = &Gban {
        user_id: user_id.unwrap(),
        reason: reason.clone(),
    };
    if (*SUDO_USERS).contains(&user_id.unwrap()) {
        cx.reply_to("I am not gonna ban a sudo user").await?;
        return Ok(());
    }
    if user_id.unwrap() == *OWNER_ID {
        cx.reply_to("I am not gonna gban my owner of all people")
            .await?;
        return Ok(());
    }
    if user_id.unwrap() == get_bot_id(cx).await {
        cx.reply_to("Haha I am not gonna ban myself fuck off!")
            .await?;
        return Ok(());
    }
    if is_gbanned(&db, &user_id.unwrap()).await? {
        if reason.is_empty() {
            cx.reply_to("This user is already gbanned I would have loved to change the reason for his gban but you haven't given me one").await?;
            return Ok(());
        }
        gban_user(&db, gb).await?;
        return Ok(());
    }
    let msg = cx.reply_to("Gbanning the user...").await?;
    gban_user(&db, gb).await?;
    let chats = get_all_chats(&db).await?;
    for c in chats {
        if let Ok(chatmem) = cx.requester.get_chat_member(c, user_id.unwrap()).await {
            if chatmem.is_privileged() || chatmem.is_banned() {
                continue;
            }
            if cx
                .requester
                .kick_chat_member(c, user_id.unwrap())
                .await
                .is_err()
            {
                continue;
            }
        }
    }
    cx.requester
        .edit_message_text(cx.chat_id(), msg.id, "Gbanned the fucker")
        .await?;
    Ok(())
}

pub async fn ungban(cx: &Cxt) -> TgErr<()> {
    let (user_id, _) = extract_text_id_from_reply(cx).await;
    if user_id.is_none() {
        cx.reply_to("Refer someone to gban").await?;
        return Ok(());
    }
    let db = get_mdb().await;
    if (*SUDO_USERS).contains(&user_id.unwrap()) {
        cx.reply_to("Pretty sure this user isn't gbanned because he's my sudo user")
            .await?;
        return Ok(());
    }
    if user_id.unwrap() == *OWNER_ID {
        cx.reply_to("This user is not gbanned because he's my owner")
            .await?;
        return Ok(());
    }
    if user_id.unwrap() == get_bot_id(cx).await {
        cx.reply_to("The day I gban myself is the release date of Halflife 3")
            .await?;
        return Ok(());
    }
    if is_gbanned(&db, &user_id.unwrap()).await? {
        ungban_user(&db, &user_id.unwrap()).await?;
        let chats = get_all_chats(&db).await?;
        let msg = cx.reply_to("Ungbanning the poor fucker").await?;
        for c in chats {
            if let Ok(mem) = cx.requester.get_chat_member(c, user_id.unwrap()).await {
                if mem.is_banned()
                    && cx
                        .requester
                        .unban_chat_member(c, user_id.unwrap())
                        .await
                        .is_err()
                {
                    continue;
                }
            }
        }
        cx.requester
            .edit_message_text(cx.chat_id(), msg.id, "Ungban Completed")
            .await?;
    } else {
        cx.reply_to("Why are you trying to ungban the user who's not even gbanned")
            .await?;
    }
    Ok(())
}

pub async fn gbanstat(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id),
    )?;
    let (_, args) = parse_command(cx.update.text().unwrap(), "consts::BOT_NAME").unwrap();
    let db = get_mdb().await;
    if args.is_empty() {
        cx.reply_to("What should I do with this?").await?;
        return Ok(());
    }
    let gstat = args[0].to_lowercase().parse::<GbanStats>().unwrap();
    match gstat {
        GbanStats::On => {
            let gs = &GbanStat {
                chat_id: cx.chat_id(),
                is_on: true,
            };
            set_gbanstat(&db, gs).await?;
            cx.reply_to(
                "I have enabled Gbans for this chat you will be more protected from trolls now",
            )
            .await?;
        }
        GbanStats::Off => {
            let gs = &GbanStat {
                chat_id: cx.chat_id(),
                is_on: false,
            };
            set_gbanstat(&db, gs).await?;
            cx.reply_to(
                "I have disabled Gbans for this chat you will be less protected from trolls though",
            )
            .await?;
        }
        GbanStats::Error => {
            cx.reply_to("That's an invalid gbanstat input valid one's are (yes/on),(no,off)")
                .await?;
        }
    }
    Ok(())
}
