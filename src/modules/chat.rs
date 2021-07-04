use crate::{
    util::{can_change_info, consts, get_bot_id, get_filter_type, FilterType},
    Cxt, TgErr,
};
use teloxide::{
    net::Download,
    payloads::SendMessageSetters,
    prelude::{GetChatId, Requester},
    types::{ChatKind, InputFile, ParseMode},
    utils::command::parse_command,
    utils::html,
};
use tokio::{fs, try_join};

pub async fn set_chatpic(cx: &Cxt) -> TgErr<()> {
    let bot_id = get_bot_id(cx).await;
    tokio::try_join!(
        can_change_info(cx, cx.update.from().unwrap().id),
        can_change_info(cx, bot_id)
    )?;
    if cx.update.reply_to_message().is_none() {
        cx.reply_to("Reply to a picture to set it as a chat picture")
            .await?;
        return Ok(());
    }

    let f = cx.update.reply_to_message().unwrap();

    //Get the message type
    let m_type = get_filter_type(f).await.parse::<FilterType>().unwrap();
    //Image path abs{chat_id}.png
    let path = format!("{}.png", cx.chat_id().abs());
    if let ChatKind::Public(c) = &cx.update.chat.kind {
        let title = c.title.as_ref().unwrap();
        match m_type {
            FilterType::Document => {
                if matches!(
                    f.document().unwrap().mime_type.as_ref().unwrap().type_(),
                    mime::IMAGE
                ) {
                    let file_id = f.document().as_ref().unwrap().file_id.as_str();
                    let file_path = cx.requester.get_file(file_id).await?.file_path;
                    let mut file = fs::File::create(&path).await?;
                    cx.requester.download_file(&file_path, &mut file).await?;
                    let doc = InputFile::file(&path);
                    cx.requester.set_chat_photo(cx.chat_id(), doc).await?;
                    cx.reply_to(format!("Chat picture updated for chat {}", title))
                        .await?;
                } else {
                    cx.reply_to("This type is not supported").await?;
                }
            }
            FilterType::Photos => {
                //Last photo in the vector is the image with best quality
                let file_id = f.photo().as_ref().unwrap().last().unwrap().file_id.as_str();
                let file_path = cx.requester.get_file(file_id).await?.file_path;
                let mut file = fs::File::create(&path).await?;
                cx.requester.download_file(&file_path, &mut file).await?;
                let photo = InputFile::file(&path);
                cx.requester.set_chat_photo(cx.chat_id(), photo).await?;
                cx.reply_to(format!("Chat picture updated for chat {}", title))
                    .await?;
            }
            _ => {
                cx.reply_to("Reply to an image to set it as chat picture")
                    .await?;
            }
        }
    } else {
        cx.reply_to("This command can only be used in a group")
            .await?;
    }
    //If the downloaded chat pic still exists delet
    if fs::metadata(&path).await.is_ok() {
        fs::remove_file(&path).await?;
    }

    Ok(())
}

pub async fn set_chat_tile(cx: &Cxt) -> TgErr<()> {
    try_join!(
        can_change_info(cx, cx.update.from().unwrap().id),
        can_change_info(cx, get_bot_id(cx).await)
    )?;
    let (_, args) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();
    let mut title;
    //If args is empty look for reply to message
    if args.is_empty() {
        if cx.update.reply_to_message().is_some() {
            //Check if the reply_to_message as a text if not check if the message has any captions
            title = cx
                .update
                .reply_to_message()
                .unwrap()
                .text()
                .unwrap_or_else(|| {
                    cx.update
                        .reply_to_message()
                        .unwrap()
                        .caption()
                        .unwrap_or("")
                })
                .to_string();
        } else {
            cx.reply_to("Provide me a title to set").await?;
            return Ok(());
        }
    } else {
        title = args.join(" ");
    }
    //Telegram has 255 character limit on the chat title (https://core.telegram.org/bots/api#setchattitle)
    if title.len() > 255 {
        cx.reply_to("The text has more than 255 characters so truncating it to 255 characters")
            .await?;
        title.truncate(255);
    }

    //Check if the title is empty possible when the user replies to a non-text message without caption
    if title.is_empty() {
        cx.reply_to("Title can't be empty").await?;
        return Ok(());
    }

    cx.requester.set_chat_title(cx.chat_id(), &title).await?;
    cx.reply_to(format!(
        "Successfully set the chat title to '{}'",
        html::code_inline(&title)
    ))
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
