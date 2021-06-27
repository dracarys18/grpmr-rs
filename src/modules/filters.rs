use regex::RegexBuilder;
use teloxide::{payloads::{SendAnimationSetters, SendAudioSetters, SendDocumentSetters, SendMessageSetters, SendPhotoSetters, SendStickerSetters, SendVideoSetters, SendVoiceSetters}, prelude::{GetChatId, Requester}, types::ParseMode, types::{InputFile, Message}, utils::command::parse_command};

use crate::{
    database::{
        db_utils::{add_filter, get_reply_filter, get_reply_type_filter, list_filters, rm_filter},
        Filters,
    },
    get_mdb,
    util::{is_group, user_should_be_admin, FilterType},
};
use crate::{Cxt, TgErr};

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

pub async fn extract_text(msg: &Message) -> Option<String> {
    if msg.caption().is_some() {
        return msg.caption().map(|s| s.to_string());
    } else if msg.text().is_some() {
        return msg.text().map(|s| s.to_string());
    } else if msg.sticker().is_some() {
        return msg.sticker().map(|s| s.clone().emoji.unwrap());
    }
    None
}
pub async fn filter(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id)
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
    if !args.is_empty() {
        let keyword = args[0].to_string();
        if args.get(1).is_none() {
            if cx.update.reply_to_message().is_some() {
                let rep_msg = cx.update.reply_to_message().unwrap();
                let fil_type = get_filter_type(&rep_msg).await;
                let parsed_type = fil_type.parse::<FilterType>().unwrap();
                let reply;
                match parsed_type {
                    FilterType::Text => reply = rep_msg.text().unwrap().to_string(),
                    FilterType::Animation => {
                        reply = rep_msg.animation().unwrap().file_id.to_string()
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
                f_type: "text".to_string(),
            };
            add_filter(&db, fl).await?;
            cx.reply_to(format!("Saved filter <code>'{}'</code>", &keyword))
                .parse_mode(ParseMode::Html)
                .await?;
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
    let (_, args) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
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
    Ok(())
}

pub async fn filter_reply(cx: &Cxt) -> TgErr<()> {
    let db = get_mdb().await;
    let text = extract_text(&cx.update).await;
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
            match parsed_ftype {
                FilterType::Audio => {
                    let audio = InputFile::file_id(reply);
                    cx.requester.send_audio(cx.chat_id(),audio).reply_to_message_id(cx.update.id).await?;
                }
                FilterType::Animation => {
                    let animation = InputFile::file_id(reply);
                    cx.requester.send_animation(cx.chat_id(),animation).reply_to_message_id(cx.update.id).await?;
                }
                FilterType::Document => {
                    let document = InputFile::file_id(reply);
                    cx.requester.send_document(cx.chat_id(),document).reply_to_message_id(cx.update.id).await?;
                }
                FilterType::Photos => {
                    let photo = InputFile::file_id(reply);
                    cx.requester.send_photo(cx.chat_id(),photo).reply_to_message_id(cx.update.id).await?;
                }
                FilterType::Sticker => {
                    let sticker = InputFile::file_id(reply);
                    cx.requester.send_sticker(cx.chat_id(),sticker).reply_to_message_id(cx.update.id).await?;
                }
                FilterType::Text => {
                    cx.reply_to(reply).await?;
                }
                FilterType::Video => {
                    let video = InputFile::file_id(reply);
                    cx.requester.send_video(cx.chat_id(),video).reply_to_message_id(cx.update.id).await?;
                }
                FilterType::Voice => {
                    let voice = InputFile::file_id(reply);
                    cx.requester.send_voice(cx.chat_id(), voice).reply_to_message_id(cx.update.id).await?;
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
    cx.reply_to(format!(
        "<code>Filters in this group are</code>:\n- {}",
        filters.join("\n- ")
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
