use teloxide::prelude::GetChatId;
use teloxide::utils::command::parse_command;

use crate::database::db_utils::{disable_command, get_disabled_command};
use crate::database::DisableCommand;
use crate::util::{consts, is_group, user_should_be_admin, DisableAble};
use crate::{get_mdb, Cxt, TgErr};

pub async fn disable(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id)
    )?;
    let db = get_mdb().await;
    let (_, args) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();

    if args.is_empty() {
        cx.reply_to("What should I disable").await?;
        return Ok(());
    }
    let disable_val = args[0].to_lowercase();
    let dis_enu = disable_val.parse::<DisableAble>().unwrap();
    let mut cmds = get_disabled_command(&db, cx.chat_id()).await?;
    if !matches!(dis_enu, DisableAble::Error) {
        cmds.push(disable_val.clone());
        let dc = &DisableCommand {
            chat_id: cx.chat_id(),
            disabled_commands: cmds,
        };
        disable_command(&db, dc).await?;
        cx.reply_to(format!("Command {} has been disabled", disable_val))
            .await?;
    } else {
        cx.reply_to("This command Can't be disabled!").await?;
    }
    Ok(())
}

pub async fn enable(cx: &Cxt) -> TgErr<()> {
    tokio::try_join!(
        is_group(cx),
        user_should_be_admin(cx, cx.update.from().unwrap().id)
    )?;
    let (_, args) = parse_command(cx.update.text().unwrap(), consts::BOT_NAME).unwrap();
    let db = get_mdb().await;
    if args.is_empty() {
        cx.reply_to("What should I disable").await?;
        return Ok(());
    }
    let cmd = args[0].to_lowercase();
    let mut disabled_cmds = get_disabled_command(&db, cx.chat_id()).await?;
    let ind = disabled_cmds.iter().position(|pos| pos.eq(&cmd));
    if ind.is_some() {
        disabled_cmds.remove(ind.unwrap());
        let dc = &DisableCommand {
            chat_id: cx.chat_id(),
            disabled_commands: disabled_cmds,
        };
        disable_command(&db, dc).await?;
        cx.reply_to("This command is enabled and ready to use here")
            .await?;
    } else {
        cx.reply_to("Try enabling something which was disabled")
            .await?;
    }
    Ok(())
}
