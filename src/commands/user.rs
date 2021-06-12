use crate::util::extract_text_id_from_reply;
use crate::{Cxt, Err};
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::html::user_mention;

pub async fn info(cx: &Cxt) -> Err {
    let (user_id, _text) = extract_text_id_from_reply(cx).await;
    let chat = cx.update.chat.clone();
    let mut user = None;
    if user_id.is_some() && chat.is_group() && chat.is_supergroup() {
        user = Some(
            cx.requester
                .get_chat_member(cx.chat_id(), user_id.unwrap())
                .await
                .unwrap()
                .user,
        );
    } else if cx.update.reply_to_message().is_none() && _text.is_none() {
        user = Some(cx.update.from().unwrap().to_owned());
    } else if user_id.is_none() {
        cx.reply_to("This user is either dead or not in my database!")
            .await?;
        return Ok(());
    }
    if user.is_none() {
        cx.reply_to("Can't seem to get info about the user").await?;
        return Ok(());
    }

    let us_inf = user.unwrap();
    let mut info_text = format!(
        "<b>User info</b>:\nID:<code>{}</code>\nFirst Name: {}",
        &us_inf.id, &us_inf.first_name
    );

    if let Some(ln) = us_inf.clone().last_name {
        info_text = format!("{}\nLast Name: {}", info_text, ln);
    }

    if let Some(un) = us_inf.clone().username {
        info_text = format!("{}\nUsername: @{}", info_text, un);
    }

    info_text = format!(
        "{}\nPermanent Link: {}",
        info_text,
        user_mention(us_inf.id as i32, "link")
    );

    cx.reply_to(info_text).parse_mode(ParseMode::Html).await?;
    return Ok(());
}
