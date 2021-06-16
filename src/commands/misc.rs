use crate::{Cxt, TgErr};
use regex::Regex;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
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
    .await?
    .text()
    .await?;
    let data: serde_json::Value = serde_json::from_str(&resp).unwrap();
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
    let mut reply_text = ignore_char.replace_all(&txt, "").into_owned();
    //Telegram's character limit
    reply_text.truncate(4096);
    cx.reply_to(reply_text).parse_mode(ParseMode::Html).await?;
    Ok(())
}
