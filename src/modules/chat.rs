use crate::{
    util::{can_change_info, get_bot_id, get_filter_type, FilterType},
    Cxt, TgErr,
};
use teloxide::{
    net::Download,
    prelude::{GetChatId, Requester},
    types::{ChatKind, InputFile},
};
use tokio::fs;

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
