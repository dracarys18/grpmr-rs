use crate::database::{Chat, Gban, User};
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
        return Ok(us_id);
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
