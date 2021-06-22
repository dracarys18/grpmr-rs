pub struct Db {
    db_uri: String,
}

impl Db {
    pub fn new(uri: String) -> Self {
        Db { db_uri: uri }
    }
    pub async fn client(self) -> mongodb::Database {
        let conf = mongodb::options::ClientOptions::parse(self.db_uri)
            .await
            .unwrap();
        let client = mongodb::Client::with_options(conf).unwrap();
        client.database("tgbot")
    }
}
