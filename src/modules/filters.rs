use regex::RegexBuilder;
use teloxide::{
    payloads::{
        SendAnimationSetters, SendAudioSetters, SendDocumentSetters, SendMessageSetters,
        SendPhotoSetters, SendVideoSetters, SendVoiceSetters,
    },
    prelude::{GetChatId, Requester},
    types::InputFile,
    types::ParseMode,
    utils::{
        command::parse_command,
        html::{self, user_mention_or_link},
    },
};

use crate::{
    database::{
        db_utils::{
            add_blacklist, add_filter, get_blacklist, get_blacklist_mode, get_log_channel,
            get_reply_caption, get_reply_filter, get_reply_type_filter, list_filters, rm_blacklist,
            rm_filter, set_blacklist_mode,
        },
        BlacklistFilter, BlacklistKind, Filters,
    },
    get_mdb,
    modules::{send_log, warn_user},
    util::{
        can_delete_messages, consts, extract_filter_text, get_bot_id, get_chat_title,
        get_filter_type, is_group, is_user_admin, user_should_be_admin, user_should_restrict,
        BlacklistMode, FilterType,
    },
};
use crate::{Cxt, TgErr};

pub async fn filter(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id)
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();
    if !args.is_empty() {
        let keyword = args[0].to_string();
        if args.get(1).is_none() {
            if cx.update.reply_to_message().is_some() {
                let rep_msg = cx.update.reply_to_message().unwrap();
                let fil_type = get_filter_type(rep_msg).await;
                let parsed_type = fil_type.parse::<FilterType>().unwrap();
                let reply;
                let cap = rep_msg.caption().map(String::from);
                match parsed_type {
                    FilterType::Text => reply = rep_msg.text().unwrap().to_string(),
                    FilterType::Animation => {
                        reply = rep_msg.animation().unwrap().file_id.to_string();
                    }
                    FilterType::Audio => reply = rep_msg.audio().unwrap().file_id.to_string(),
                    FilterType::Document => reply = rep_msg.document().unwrap().file_id.to_string(),
                    FilterType::Photos => {
                        reply = rep_msg.photo().unwrap().last().unwrap().file_id.to_string()
                    }
                    FilterType::Sticker => reply = rep_msg.sticker().unwrap().file_id.to_string(),
                    FilterType::Video => reply = rep_msg.video().unwrap().file_id.to_string(),
                    FilterType::Voice => reply = rep_msg.voice().unwrap().file_id.to_string(),
                    FilterType::Error => {
                        cx.reply_to("This filter type is not supported").await?;
                        return Ok(());
                    }
                }
                let fl = &Filters {
                    chat_id: cx.chat_id(),
                    filter: keyword.clone(),
                    reply: reply.clone(),
                    caption: cap,
                    f_type: fil_type,
                };
                add_filter(&db, fl).await?;
                cx.reply_to(format!("Saved filter <code>'{}'</code>", &keyword))
                    .parse_mode(ParseMode::Html)
                    .await?;
            } else {
                cx.reply_to("Give me something to reply the filter with")
                    .await?;
            }
        } else {
            let rep = args[1..].join("");
            let fl = &Filters {
                chat_id: cx.chat_id(),
                filter: keyword.clone(),
                reply: rep,
                caption: None,
                f_type: "text".to_string(),
            };
            add_filter(&db, fl).await?;
            cx.reply_to(format!("Saved filter <code>'{}'</code>", &keyword))
                .parse_mode(ParseMode::Html)
                .await?;
            if let Some(l) = get_log_channel(&db, cx.chat_id()).await? {
                let admin = cx
                    .requester
                    .get_chat_member(cx.chat_id(), cx.update.from().unwrap().id)
                    .await?
                    .user;
                let logm = format!(
                    "Chat title: {}\n#FILTER\nAdmin: {}\nWord: {}",
                    html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
                    html::user_mention(admin.id, &admin.full_name()),
                    html::code_inline(&keyword)
                );
                send_log(cx, &logm, l).await?;
            }
        }
    } else {
        cx.reply_to("Give me some keyword to filter").await?;
    }
    Ok(())
}

pub async fn remove_filter(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id),
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();
    if args.is_empty() {
        cx.reply_to("Mention some filter keyword to stop").await?;
        return Ok(());
    }
    let keyword = args[0].to_owned();
    let filist = list_filters(&db, cx.chat_id()).await?;
    if !filist.contains(&keyword) {
        cx.reply_to("You haven't set any filter on that keyword")
            .await?;
        return Ok(());
    }
    rm_filter(&db, cx.chat_id(), &keyword).await?;
    cx.reply_to(format!(
        "Filter <code>{}</code> has been stopped.",
        &keyword
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    if let Some(l) = get_log_channel(&db, cx.chat_id()).await? {
        let admin = cx
            .requester
            .get_chat_member(cx.chat_id(), cx.update.from().unwrap().id)
            .await?
            .user;
        let logm = format!(
            "Chat title: {}\n#FILTER_STOPPED\nAdmin: {}\nWord: {}",
            html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
            html::user_mention(admin.id, &admin.full_name()),
            html::code_inline(&keyword)
        );
        send_log(cx, &logm, l).await?;
    }
    Ok(())
}

pub async fn filter_reply(cx: &Cxt) -> TgErr<()> {
    let db = get_mdb().await;
    let text = extract_filter_text(&cx.update).await;
    if text.is_none() {
        return Ok(());
    }
    let to_match = text.unwrap();
    let filterlist = list_filters(&db, cx.chat_id()).await?;
    for pat in filterlist {
        let pattern = format!(r#"( |^|[^\w]){}( |$|[^\w])"#, regex::escape(&pat));
        let re = RegexBuilder::new(&pattern)
            .case_insensitive(true)
            .build()
            .unwrap();
        if re.is_match(&to_match) {
            let f_type = get_reply_type_filter(&db, cx.chat_id(), &pat)
                .await?
                .unwrap();
            let reply = get_reply_filter(&db, cx.chat_id(), &pat).await?.unwrap();
            let parsed_ftype = f_type.parse::<FilterType>().unwrap();
            let caption = get_reply_caption(&db, cx.chat_id(), &pat)
                .await?
                .unwrap_or_else(String::new);
            match parsed_ftype {
                FilterType::Audio => {
                    let audio = InputFile::file_id(reply);
                    cx.reply_audio(audio).caption(caption).await?;
                }
                FilterType::Animation => {
                    let animation = InputFile::file_id(reply);
                    cx.reply_animation(animation).caption(caption).await?;
                }
                FilterType::Document => {
                    let document = InputFile::file_id(reply);
                    cx.reply_document(document).caption(caption).await?;
                }
                FilterType::Photos => {
                    let photo = InputFile::file_id(reply);
                    cx.reply_photo(photo).caption(caption).await?;
                }
                FilterType::Sticker => {
                    let sticker = InputFile::file_id(reply);
                    cx.reply_sticker(sticker).await?;
                }
                FilterType::Text => {
                    cx.reply_to(reply).await?;
                }
                FilterType::Video => {
                    let video = InputFile::file_id(reply);
                    cx.reply_video(video).caption(caption).await?;
                }
                FilterType::Voice => {
                    let voice = InputFile::file_id(reply);
                    cx.reply_voice(voice).caption(caption).await?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
pub async fn filter_list(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id),
    )?;
    let db = get_mdb().await;
    let filters = list_filters(&db, cx.chat_id()).await?;
    if filters.is_empty() {
        cx.reply_to(format!(
            "No filters in {}",
            get_chat_title(cx, cx.chat_id()).await.unwrap()
        ))
        .await?;
        return Ok(());
    }
    cx.reply_to(format!(
        "<code>Filters in this group are</code>:\n- {}",
        filters.join("\n- ")
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn blacklist_filter(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_restrict(cx, cx.update.from().unwrap().id),
        user_should_restrict(cx, get_bot_id(cx).await)
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();
    if args.is_empty() {
        cx.reply_to("What should I blacklist").await?;
        return Ok(());
    }
    let blacklist = args[0].to_owned();
    let bl = &BlacklistFilter {
        chat_id: cx.chat_id(),
        filter: blacklist.clone(),
    };
    add_blacklist(&db, bl).await?;
    let mode = get_blacklist_mode(&db, cx.chat_id()).await?;

    //Because the default mode is delete we need to check for the permissions for other modes bot will take care about the permissions while setting the modes
    if matches!(
        mode.parse::<BlacklistMode>().unwrap(),
        BlacklistMode::Delete
    ) {
        can_delete_messages(cx, get_bot_id(cx).await).await?;
    }

    cx.reply_to(format!(
        "Added blacklist {}. The blacklist mode in the chat is <code>{}</code>",
        &blacklist, &mode
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    if let Some(l) = get_log_channel(&db, cx.chat_id()).await? {
        let admin = cx
            .requester
            .get_chat_member(cx.chat_id(), cx.update.from().unwrap().id)
            .await?
            .user;
        let logm = format!(
            "Chat title: {}\n#BLACKLIST\nAdmin: {}\nWord: {}",
            html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
            html::user_mention(admin.id, &admin.full_name()),
            html::code_inline(&blacklist)
        );
        send_log(cx, &logm, l).await?;
    }
    Ok(())
}

pub async fn list_blacklist(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id)
    )?;
    let db = get_mdb().await;
    let blist = get_blacklist(&db, cx.chat_id()).await?;
    if blist.is_empty() {
        cx.reply_to("No blacklist set in this chat").await?;
        return Ok(());
    }
    cx.reply_to(format!(
        "The blacklist words in the chat are\n <code>- {}</code>",
        blist.join("\n -")
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn remove_blacklist(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id)
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), "consts::BOT_NAME").unwrap();
    if args.is_empty() {
        cx.reply_to("Mention some blacklist to remove").await?;
        return Ok(());
    }
    let bk = args[0].to_string();
    let blist = get_blacklist(&db, cx.chat_id()).await?;
    if blist.contains(&bk) {
        let bl = &BlacklistFilter {
            chat_id: cx.chat_id(),
            filter: bk.clone(),
        };
        rm_blacklist(&db, bl).await?;
        cx.reply_to(format!("Blacklist {} has been removed", &bk))
            .await?;
        if let Some(l) = get_log_channel(&db, cx.chat_id()).await? {
            let admin = cx
                .requester
                .get_chat_member(cx.chat_id(), cx.update.from().unwrap().id)
                .await?
                .user;
            let logm = format!(
                "Chat title: {}\n#BLACKLIST_REMOVED\nAdmin: {}\nWord: {}",
                html::code_inline(&get_chat_title(cx, cx.chat_id()).await.unwrap()),
                html::user_mention(admin.id, &admin.full_name()),
                html::code_inline(&bk)
            );
            send_log(cx, &logm, l).await?;
        }
    } else {
        cx.reply_to("This word isn't blacklisted here!").await?;
    }
    Ok(())
}

pub async fn set_blacklist_kind(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id)
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), "consts::BOT_NAME").unwrap();
    let bot_id = get_bot_id(cx).await;
    if args.is_empty() {
        cx.reply_to("Mention a blacklist mode").await?;
        return Ok(());
    }
    let mode = args[0].parse::<BlacklistMode>().unwrap();
    let chatmem = cx.requester.get_chat_member(cx.chat_id(), bot_id).await?;
    match mode {
        BlacklistMode::Warn => {
            if chatmem.kind.can_restrict_members() {
                let bm = &BlacklistKind {
                    chat_id: cx.chat_id(),
                    kind: String::from("warn"),
                };
                set_blacklist_mode(&db, bm).await?;
                cx.reply_to("The blacklist mode is set to <i>warn</i> that means if you use the blacklisted word you will be warned, when your warns exceed the warn limit you will be banned or kicked depending upon the warn mode").parse_mode(ParseMode::Html).await?;
            } else {
                cx.reply_to("I can't restrict people here please give me the permission to do so")
                    .await?;
            }
        }
        BlacklistMode::Kick => {
            if chatmem.kind.can_restrict_members() {
                let bm = &BlacklistKind {
                    chat_id: cx.chat_id(),
                    kind: String::from("kick"),
                };
                set_blacklist_mode(&db, bm).await?;
                cx.reply_to("The blacklist mode is set to <i>kick</i> that means if you use the blacklisted word you will be kicked from the group").parse_mode(ParseMode::Html).await?;
            } else {
                cx.reply_to("I can't restrict people here please give me the permission to do so")
                    .await?;
            }
        }
        BlacklistMode::Delete => {
            if chatmem.kind.can_delete_messages() {
                let bm = &BlacklistKind {
                    chat_id: cx.chat_id(),
                    kind: String::from("delete"),
                };
                set_blacklist_mode(&db, bm).await?;
                cx.reply_to("The blacklist mode is set to <i>delete</i> that means if you use the blacklisted word the message will be deleted").parse_mode(ParseMode::Html).await?;
            } else {
                cx.reply_to("I can't delete messages here please give me the permission to do so")
                    .await?;
            }
        }
        BlacklistMode::Ban => {
            if chatmem.kind.can_restrict_members() {
                let bm = &BlacklistKind {
                    chat_id: cx.chat_id(),
                    kind: String::from("ban"),
                };
                set_blacklist_mode(&db, bm).await?;
                cx.reply_to("The blacklist mode is set to <i>ban</i> that means if you use the blacklisted word you will be banned from the group").parse_mode(ParseMode::Html).await?;
            } else {
                cx.reply_to("I can't restrict people here please give me the permission to do so")
                    .await?;
            }
        }
        BlacklistMode::Error => {
            cx.reply_to("Invalid blacklist mode").await?;
        }
    }
    Ok(())
}

pub async fn action_blacklist(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),                                           //Should be a group
    )?;
    let text = extract_filter_text(&cx.update).await;
    let bot_id = get_bot_id(cx).await;
    let db = get_mdb().await;

    if cx.update.from().is_none() {
        return Ok(());
    }
    if text.is_none() {
        return Ok(());
    }
    let culprit_id = cx.update.from().unwrap().id;
    if culprit_id == bot_id {
        return Ok(());
    }
    if is_user_admin(cx, culprit_id).await {
        return Ok(());
    }
    let to_match = text.unwrap();
    let blacklist = get_blacklist(&db, cx.chat_id()).await?;
    let mode = get_blacklist_mode(&db, cx.chat_id()).await?;
    let parsed_mode = mode.parse::<BlacklistMode>().unwrap();
    let botmem = cx.requester.get_chat_member(cx.chat_id(), bot_id).await?;
    let usmem = cx
        .requester
        .get_chat_member(cx.chat_id(), culprit_id)
        .await?;
    for pat in blacklist {
        let pattern = format!(r#"( |^|[^\w]){}( |$|[^\w])"#, regex::escape(&pat));
        let re = RegexBuilder::new(&pattern)
            .case_insensitive(true)
            .build()
            .unwrap();
        if re.is_match(&to_match) {
            match parsed_mode {
                BlacklistMode::Ban => {
                    if botmem.kind.can_restrict_members() {
                        cx.requester
                            .kick_chat_member(cx.chat_id(), culprit_id)
                            .await?;
                        cx.reply_to(format!(
                            "User{} has been banned for using blacklisted word <code>{}</code>",
                            user_mention_or_link(&usmem.user),
                            &pat
                        ))
                        .parse_mode(ParseMode::Html)
                        .await?;
                    }
                }
                BlacklistMode::Delete => {
                    if botmem.kind.can_delete_messages() {
                        cx.requester
                            .delete_message(cx.chat_id(), cx.update.id)
                            .await?;
                    }
                }
                BlacklistMode::Kick => {
                    if botmem.kind.can_restrict_members() {
                        cx.requester
                            .kick_chat_member(cx.chat_id(), culprit_id)
                            .await?;
                        cx.requester
                            .unban_chat_member(cx.chat_id(), culprit_id)
                            .await?;
                        cx.reply_to(format!(
                            "User {} has been kicked for using the blacklisted word {}",
                            user_mention_or_link(&usmem.user),
                            &pat
                        ))
                        .await?;
                    }
                }
                BlacklistMode::Warn => {
                    if botmem.kind.can_restrict_members() {
                        let reason = format!("Use of blacklisted word {}", &pat);
                        warn_user(cx, culprit_id, reason).await?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
