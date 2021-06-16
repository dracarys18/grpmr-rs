use crate::util::owner_filter;
use crate::{Cxt, TgErr, OWNER_ID};
use teloxide::prelude::*;
use teloxide::types::{ChatKind, ParseMode};
use teloxide::utils::command::parse_command;
use teloxide::utils::html;

pub async fn leavechat(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(owner_filter(cx.update.from().unwrap().id),)?;
    let (_, txt) = parse_command(cx.update.text().unwrap(), "grpmr_bot").unwrap();
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
