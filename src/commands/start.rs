use crate::{Cxt, Err};
use teloxide::utils::html::user_mention_or_link;

pub async fn start_handler(cx: &Cxt) -> Err {
    let start_message_priv = format!(
        "Hello {}! Hope you are doing well",
        user_mention_or_link(cx.update.from().unwrap())
    );
    let start_message_pub = "I am alive boss!".to_owned();

    if cx.update.chat.is_private() {
        cx.reply_to(start_message_priv).await?;
        return Ok(());
    }
    cx.reply_to(start_message_pub).await?;
    return Ok(());
}
