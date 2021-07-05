use super::{
    BlacklistFilter, BlacklistKind, Chat, DisableCommand, Filters, Gban, GbanStat, Logging,
    Reporting, User, Warn, WarnKind, Warnlimit,
};
use crate::{Cxt, TgErr};
use mongodb::{bson::doc, Database};
use teloxide::prelude::StreamExt;
use teloxide::types::ChatKind;

type DbResult<T> = Result<T, mongodb::error::Error>;

fn user_collection(db: &Database) -> mongodb::Collection<User> {
    db.collection("User")
}
fn chat_collection(db: &Database) -> mongodb::Collection<Chat> {
    db.collection("Chats")
}
fn gban_collection(db: &Database) -> mongodb::Collection<Gban> {
    db.collection("Gban")
}
fn warn_collection(db: &Database) -> mongodb::Collection<Warn> {
    db.collection("Warn")
}
fn warn_kind_collection(db: &Database) -> mongodb::Collection<WarnKind> {
    db.collection("WarnKind")
}
fn warn_limit_collection(db: &Database) -> mongodb::Collection<Warnlimit> {
    db.collection("Warnlimit")
}
fn gbanstat_collection(db: &Database) -> mongodb::Collection<GbanStat> {
    db.collection("GbanStat")
}
fn disable_collection(db: &Database) -> mongodb::Collection<DisableCommand> {
    db.collection("DisableCommand")
}
fn chat_filters(db: &Database) -> mongodb::Collection<Filters> {
    db.collection("ChatFilters")
}
fn chat_blacklist(db: &Database) -> mongodb::Collection<BlacklistFilter> {
    db.collection("ChatBlacklist")
}
fn chat_blacklist_mode(db: &Database) -> mongodb::Collection<BlacklistKind> {
    db.collection("ChatBlacklistMode")
}
fn log_collection(db: &Database) -> mongodb::Collection<Logging> {
    db.collection("Logchannel")
}
fn report_collection(db: &Database) -> mongodb::Collection<Reporting> {
    db.collection("Reporting")
}
pub async fn insert_user(db: &Database, us: &User) -> DbResult<mongodb::results::UpdateResult> {
    let user = user_collection(db);
    user.update_one(
        doc! {"user_id":us.user_id},
        doc! {"$set":{"user_name":us.user_name.to_owned()}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}
pub async fn get_userid_from_name(db: &Database, username: String) -> DbResult<Option<i64>> {
    let user = user_collection(db);
    let id = user.find_one(doc! {"user_name":username}, None).await?;
    Ok(id.map(|u| u.user_id))
}

pub async fn save_user(cx: &Cxt, db: &Database) -> TgErr<()> {
    if let Some(user) = cx.update.from() {
        let uname = user.username.as_ref().map(|s| s.to_lowercase());
        let user = &User {
            user_id: user.id,
            user_name: uname.unwrap_or_else(|| "None".to_string()),
        };
        insert_user(&db, user).await?;
    }
    Ok(())
}

pub async fn insert_chat(db: &Database, c: &Chat) -> DbResult<mongodb::results::UpdateResult> {
    let chat = chat_collection(db);
    chat.update_one(
        doc! {"chat_id":&c.chat_id},
        doc! {"$set":{"chat_name":&c.chat_name}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}

pub async fn get_all_chats(db: &Database) -> DbResult<Vec<i64>> {
    let chat = chat_collection(db);
    let cursor = chat.find(None, None).await?;
    let chats: Vec<i64> = cursor.map(|chat| chat.unwrap().chat_id).collect().await;
    Ok(chats)
}
pub async fn save_chat(cx: &Cxt, db: &Database) -> TgErr<()> {
    if cx.update.chat.is_chat() {
        let chat = &cx.update.chat;
        match &chat.kind {
            ChatKind::Public(ch) => {
                let c = Chat {
                    chat_id: chat.id,
                    chat_name: ch.title.clone().unwrap(),
                };
                insert_chat(db, &c).await?;
            }
            ChatKind::Private(_) => {}
        }
    }
    Ok(())
}

pub async fn gban_user(db: &Database, gb: &Gban) -> DbResult<mongodb::results::UpdateResult> {
    let gban = gban_collection(db);
    gban.update_one(
        doc! {"user_id":&gb.user_id},
        doc! {"$set":{"reason":&gb.reason}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}

pub async fn ungban_user(db: &Database, id: &i64) -> DbResult<mongodb::results::DeleteResult> {
    let gban = gban_collection(db);
    gban.delete_one(doc! {"user_id":id}, None).await
}

pub async fn get_gban_reason(db: &Database, id: &i64) -> DbResult<String> {
    let gban = gban_collection(db);
    let reason = gban.find_one(doc! {"user_id":id}, None).await?;
    Ok(reason.map(|r| r.reason).unwrap())
}

pub async fn is_gbanned(db: &Database, id: &i64) -> DbResult<bool> {
    let gban = gban_collection(db);
    let exist = gban.find_one(doc! {"user_id":id}, None).await?;
    Ok(exist.is_some())
}

pub async fn set_gbanstat(
    db: &Database,
    gs: &GbanStat,
) -> DbResult<mongodb::results::UpdateResult> {
    let gbanstat = gbanstat_collection(db);
    gbanstat
        .update_one(
            doc! {"chat_id":gs.chat_id},
            doc! {"$set":{"is_on":gs.is_on}},
            mongodb::options::UpdateOptions::builder()
                .upsert(true)
                .build(),
        )
        .await
}

pub async fn get_gbanstat(db: &Database, id: i64) -> DbResult<bool> {
    let gbanstat = gbanstat_collection(db);
    let stat = gbanstat.find_one(doc! {"chat_id":id}, None).await?;
    if stat.is_none() {
        let gs = &GbanStat {
            chat_id: id,
            is_on: true,
        };
        set_gbanstat(db, gs).await?;
        return Ok(true);
    }
    Ok(stat.map(|d| d.is_on).unwrap())
}
pub async fn insert_warn(db: &Database, w: &Warn) -> DbResult<mongodb::results::UpdateResult> {
    let warn = warn_collection(db);
    warn.update_one(
        doc! {"chat_id":w.chat_id},
        doc! {"$set":{"user_id":w.user_id,"reason":&w.reason,"count":w.count}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}

pub async fn get_warn_count(db: &Database, chat_id: i64, user_id: i64) -> DbResult<i64> {
    let warn = warn_collection(db);
    let count = warn
        .find_one(doc! {"chat_id":chat_id,"user_id":user_id}, None)
        .await?;
    Ok(count.map(|s| s.count as i64).unwrap_or(0_i64))
}

pub async fn set_warn_limit(
    db: &Database,
    wl: &Warnlimit,
) -> DbResult<mongodb::results::UpdateResult> {
    let wc = warn_limit_collection(db);
    wc.update_one(
        doc! {"chat_id":wl.chat_id},
        doc! {"$set":{"limit":wl.limit}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}

pub async fn set_softwarn(
    db: &Database,
    wk: &WarnKind,
) -> DbResult<mongodb::results::UpdateResult> {
    let wkc = warn_kind_collection(db);
    wkc.update_one(
        doc! {"chat_id":wk.chat_id},
        doc! {"$set":{"softwarn":wk.softwarn}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}

pub async fn get_softwarn(db: &Database, id: i64) -> DbResult<bool> {
    let wkc = warn_kind_collection(db);
    let warn_kind = wkc.find_one(doc! {"chat_id":id}, None).await?;
    if warn_kind.is_none() {
        //Default
        let wk = &WarnKind {
            chat_id: id,
            softwarn: false,
        };
        set_softwarn(db, wk).await?;
        return Ok(false);
    }
    Ok(warn_kind.map(|d| d.softwarn).unwrap())
}

pub async fn get_warn_limit(db: &Database, id: i64) -> DbResult<i64> {
    let warn = warn_limit_collection(db);
    let warn_lim = warn.find_one(doc! {"chat_id":id}, None).await?;
    if warn_lim.is_none() {
        //set default limit to 3
        let wl = &Warnlimit {
            chat_id: id,
            limit: 3_u64,
        };
        set_warn_limit(db, wl).await?;
        Ok(3_i64)
    } else {
        Ok(warn_lim.map(|s| s.limit as i64).unwrap())
    }
}
pub async fn rm_single_warn(
    db: &Database,
    chat_id: i64,
    user_id: i64,
) -> DbResult<mongodb::results::UpdateResult> {
    let warn = warn_collection(db);
    let count = get_warn_count(&db, chat_id, user_id).await?;
    warn.update_one(
        doc! {"chat_id":chat_id},
        doc! {"$set":{"count":count-1}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}
pub async fn reset_warn(
    db: &Database,
    chat_id: i64,
    user_id: i64,
) -> DbResult<mongodb::results::UpdateResult> {
    let warn = warn_collection(db);
    warn.update_one(
        doc! {"chat_id":chat_id},
        doc! {"$set":{"user_id":user_id,"count":0_i64}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}

pub async fn add_filter(db: &Database, fl: &Filters) -> DbResult<mongodb::results::UpdateResult> {
    let fc = chat_filters(db);
    fc.update_one(
        doc! {"chat_id":fl.chat_id,"filter":&fl.filter},
        doc! {"$set":{"reply":&fl.reply,"f_type":&fl.f_type,"caption":&fl.caption}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}
pub async fn get_reply_filter(db: &Database, chat_id: i64, filt: &str) -> DbResult<Option<String>> {
    let fc = chat_filters(db);
    let find = fc
        .find_one(doc! {"chat_id":chat_id,"filter":filt}, None)
        .await?;
    Ok(find.map(|f| (f.reply)))
}

pub async fn get_reply_type_filter(
    db: &Database,
    chat_id: i64,
    filt: &str,
) -> DbResult<Option<String>> {
    let fc = chat_filters(db);
    let find = fc
        .find_one(doc! {"chat_id":chat_id,"filter":filt}, None)
        .await?;
    Ok(find.map(|f| f.f_type))
}

pub async fn get_reply_caption(
    db: &Database,
    chat_id: i64,
    filt: &str,
) -> DbResult<Option<String>> {
    let fc = chat_filters(db);
    let find = fc
        .find_one(doc! {"chat_id":chat_id,"filter":filt}, None)
        .await?;
    Ok(find.map(|f| f.caption).unwrap_or(None))
}
pub async fn list_filters(db: &Database, chat_id: i64) -> DbResult<Vec<String>> {
    let fc = chat_filters(db);
    let dist = fc
        .distinct("filter", doc! {"chat_id":chat_id}, None)
        .await?;
    Ok(dist
        .iter()
        .map(|s| s.as_str().unwrap().to_owned())
        .collect())
}
pub async fn rm_filter(
    db: &Database,
    chat_id: i64,
    fit: &str,
) -> DbResult<mongodb::results::DeleteResult> {
    let fc = chat_filters(db);
    fc.delete_one(doc! {"chat_id":chat_id,"filter":fit}, None)
        .await
}
pub async fn add_blacklist(
    db: &Database,
    bl: &BlacklistFilter,
) -> DbResult<mongodb::results::UpdateResult> {
    let blc = chat_blacklist(db);
    blc.update_one(
        doc! {"chat_id":bl.chat_id},
        doc! {"$set":{"blacklist":&bl.filter}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}
pub async fn get_blacklist(db: &Database, chat_id: i64) -> DbResult<Vec<String>> {
    let blc = chat_blacklist(db);
    let find = blc
        .distinct("blacklist", doc! {"chat_id":chat_id}, None)
        .await?;
    Ok(find
        .iter()
        .map(|b| b.as_str().unwrap().to_owned())
        .collect())
}
pub async fn rm_blacklist(
    db: &Database,
    bl: &BlacklistFilter,
) -> DbResult<mongodb::results::DeleteResult> {
    let blc = chat_blacklist(db);
    blc.delete_one(doc! {"chat_id":bl.chat_id,"blacklist":&bl.filter}, None)
        .await
}
pub async fn set_blacklist_mode(
    db: &Database,
    bm: &BlacklistKind,
) -> DbResult<mongodb::results::UpdateResult> {
    let blc = chat_blacklist_mode(db);
    blc.update_one(
        doc! {"chat_id":bm.chat_id},
        doc! {"$set":{"kind":&bm.kind}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}
pub async fn get_blacklist_mode(db: &Database, id: i64) -> DbResult<String> {
    let blc = chat_blacklist_mode(db);
    let fi = blc.find_one(doc! {"chat_id":id}, None).await?;
    if fi.is_none() {
        let blk = &BlacklistKind {
            chat_id: id,
            kind: String::from("delete"),
        };
        set_blacklist_mode(db, blk).await?;
        return Ok(String::from("delete"));
    }
    Ok(fi.map(|b| b.kind).unwrap())
}
pub async fn disable_command(
    db: &Database,
    dc: &DisableCommand,
) -> DbResult<mongodb::results::UpdateResult> {
    let disable_coll = disable_collection(db);
    disable_coll
        .update_one(
            doc! {"chat_id":dc.chat_id},
            doc! {"$set":{"disabled_commands": dc.disabled_commands.clone()}},
            mongodb::options::UpdateOptions::builder()
                .upsert(true)
                .build(),
        )
        .await
}

pub async fn get_disabled_command(db: &Database, id: i64) -> DbResult<Vec<String>> {
    let dc = disable_collection(db);
    let f = dc.find_one(doc! {"chat_id":id}, None).await?;
    if f.is_none() {
        let dc = &DisableCommand {
            chat_id: id,
            disabled_commands: Vec::new(),
        };
        disable_command(&db, dc).await?;
        return Ok(Vec::new());
    }

    Ok(
        f.map(|d| d.disabled_commands.iter().map(|b| b.to_owned()).collect())
            .unwrap(),
    )
}

pub async fn add_log_channel(
    db: &Database,
    lg: &Logging,
) -> DbResult<mongodb::results::UpdateResult> {
    let lc = log_collection(db);
    lc.update_one(
        doc! {"chat_id":lg.chat_id},
        doc! {"$set":{"channel":lg.channel}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}

pub async fn rm_log_channel(db: &Database, id: i64) -> DbResult<mongodb::results::DeleteResult> {
    let lc = log_collection(db);
    lc.delete_one(doc! {"chat_id":id}, None).await
}

pub async fn get_log_channel(db: &Database, id: i64) -> DbResult<Option<i64>> {
    let lc = log_collection(db);
    let fi = lc.find_one(doc! {"chat_id":id}, None).await?;
    Ok(fi.map(|l| l.channel))
}

pub async fn set_report_setting(
    db: &Database,
    r: &Reporting,
) -> DbResult<mongodb::results::UpdateResult> {
    let rc = report_collection(db);
    rc.update_one(
        doc! {"chat_id":r.chat_id},
        doc! {"$set":{"allowed":r.allowed}},
        mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build(),
    )
    .await
}

pub async fn get_report_setting(db: &Database, chat_id: i64) -> DbResult<bool> {
    let rc = report_collection(db);
    let find = rc.find_one(doc! {"chat_id":chat_id}, None).await?;
    Ok(find.map(|r| r.allowed).unwrap_or(false))
}
