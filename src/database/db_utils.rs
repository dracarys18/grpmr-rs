use super::{Chat, Gban, GbanStat, User, Warn, WarnKind, Warnlimit};
use crate::{Cxt, TgErr};
use mongodb::{bson::doc, bson::Bson, Database};
use teloxide::prelude::StreamExt;
use teloxide::types::ChatKind;

type DbResult<T> = Result<T, mongodb::error::Error>;

fn user_collection(db: &Database) -> mongodb::Collection {
    db.collection("User")
}
fn chat_collection(db: &Database) -> mongodb::Collection {
    db.collection("Chats")
}
fn gban_collection(db: &Database) -> mongodb::Collection {
    db.collection("Gban")
}
fn warn_collection(db: &Database) -> mongodb::Collection {
    db.collection("Warn")
}
fn warn_kind_collection(db: &Database) -> mongodb::Collection {
    db.collection("WarnKind")
}
fn warn_limit_collection(db: &Database) -> mongodb::Collection {
    db.collection("Warnlimit")
}
fn gbanstat_collection(db: &Database) -> mongodb::Collection {
    db.collection("GbanStat")
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
    if id.is_none() {
        Ok(None)
    } else {
        let us_id = id.unwrap().get("user_id").unwrap().as_i64();
        Ok(us_id)
    }
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
    let chats: Vec<i64> = cursor
        .map(|chat| chat.unwrap().get("chat_id").and_then(Bson::as_i64).unwrap())
        .collect()
        .await;
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
    Ok(reason
        .map(|r| r.get("reason").and_then(Bson::as_str).unwrap().to_string())
        .unwrap())
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
    Ok(stat
        .map(|d| d.get("is_on").and_then(Bson::as_bool).unwrap())
        .unwrap())
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
    if count.is_none() {
        Ok(0_i64)
    } else {
        Ok(count
            .map(|s| s.get("count").and_then(Bson::as_i64).unwrap())
            .unwrap())
    }
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
    Ok(warn_kind
        .map(|d| d.get("softwarn").and_then(Bson::as_bool).unwrap())
        .unwrap())
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
        Ok(warn_lim
            .map(|s| s.get("limit").and_then(Bson::as_i64).unwrap())
            .unwrap())
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
