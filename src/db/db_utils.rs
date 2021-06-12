use crate::db::{Chat, User};
use crate::{Cxt, Err};
use mongodb::{bson::doc, Database};
use teloxide::types::ChatKind;

type DbResult<T> = Result<T, mongodb::error::Error>;

fn user_collection(db: &Database) -> mongodb::Collection {
    db.collection("User")
}
fn chat_collection(db: &Database) -> mongodb::Collection {
    db.collection("Chats")
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

pub async fn save_user(cx: &Cxt, db: &Database) -> Err {
    if let Some(user) = cx.update.from() {
        let uname = user.username.as_ref().map(|s| s.to_lowercase());
        let user = &User {
            user_id: user.id,
            user_name: uname.unwrap(),
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

pub async fn save_chat(cx: &Cxt, db: &Database) -> Err {
    if cx.update.chat.is_chat() {
        let chat = &cx.update.chat;
        match &chat.kind {
            ChatKind::Public(ch) => {
                let c = Chat {
                    chat_id: chat.id,
                    chat_name: ch.title.clone().unwrap().to_owned(),
                };
                insert_chat(db, &c).await?;
            }
            ChatKind::Private(_) => {}
        }
    }
    Ok(())
}
