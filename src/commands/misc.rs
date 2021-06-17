use crate::{Cxt, TgErr, BOT_TOKEN};
use mime;
use regex::Regex;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use teloxide::utils::command::parse_command;

pub async fn ud(cx: &Cxt) -> TgErr<()> {
    let (_, args) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
    if args.is_empty() {
        cx.reply_to("Please enter a keyword to search").await?;
        return Ok(());
    }
    let text = args.join("");
    let resp = reqwest::get(format!(
        "http://api.urbandictionary.com/v0/define?term={}",
        text
    ))
    .await?;
    let data: serde_json::Value = resp.json().await?;
    let ubdata = data.get("list").unwrap().get(0);
    if ubdata.is_none() {
        cx.reply_to("No result found for the keyword").await?;
        return Ok(());
    }
    let ignore_char = Regex::new(r#"[\[\]]"#).unwrap();
    let txt = format!(
        "<b>Word:</b> {}\n<b>Definition:</b>\n{}\n\n<i>Example:\n{}</i>",
        text,
        ubdata.unwrap().get("definition").unwrap().as_str().unwrap(),
        ubdata.unwrap().get("example").unwrap().as_str().unwrap()
    );
    let button = InlineKeyboardButton::url(
        "Know More".to_string(),
        ubdata
            .unwrap()
            .get("permalink")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned(),
    );
    let mut reply_text = ignore_char.replace_all(&txt, "").into_owned();
    //Telegram's character limit
    reply_text.truncate(4096);
    cx.reply_to(reply_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(InlineKeyboardMarkup::default().append_row(vec![button]))
        .await?;
    Ok(())
}

pub async fn dogbin(cx: &Cxt) -> TgErr<()> {
    let message = &cx.update;
    let (_, args) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
    let mut _data = String::new();
    if args.is_empty() {
        if let Some(m) = message.reply_to_message() {
            if let Some(txt) = m.text() {
                _data = txt.to_string();
            } else if let Some(doc) = m.document() {
                let file = doc.clone();
                let mime = file.mime_type.unwrap();
                let file_dw = cx.requester.get_file(&file.file_id).await?.file_path;
                if matches!(mime.type_(), mime::TEXT | mime::OCTET_STREAM) {
                    let url = format!(
                        "https://api.telegram.org/file/bot{}/{}",
                        *BOT_TOKEN, file_dw
                    );
                    _data = reqwest::get(url).await?.text().await?;
                }
                else {
                    cx.reply_to("Invalid file type").await?;
                    return Ok(());
                }
            }
            else{
                cx.reply_to("This file format is not supported").await?;
                return Ok(());
            }
        } else {
            cx.reply_to("No data was provided to paste").await?;
            return Ok(());
        }
    } else {
        _data = args.join("");
    }
    let client = reqwest::Client::new();
    let resp = client
        .post("https://del.dog/documents")
        .body(_data)
        .send()
        .await?;
    let status = resp.status().as_str().to_owned();
    let json: serde_json::Value = resp.json().await?;
    if status.ne("200") {
        cx.reply_to(json.get("message").unwrap().as_str().unwrap())
            .await?;
        return Ok(());
    }
    let key = json.get("key").unwrap().as_str().unwrap();
    let reply_text = format!(
        "<b>I have pasted that for you</b>\n\nhttps://del.dog/{}",
        &key
    );
    cx.reply_to(reply_text).parse_mode(ParseMode::Html).await?;
    Ok(())
}
