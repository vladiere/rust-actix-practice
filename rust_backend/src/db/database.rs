use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{Error, Surreal};

use crate::models::users::Users;

#[derive(Clone)]
pub struct Database {
    pub client: Surreal<Client>,
    pub name_space: String,
    pub db_name: String,
}

impl Database {
    pub async fn init() -> Result<Self, Error> {
        let client = Surreal::new::<Ws>("127.0.0.0:9000").await?;
        client
            .signin(Root {
                username: "root",
                password: "root",
            })
            .await?;

        client.use_ns("surreal").use_db("users").await.unwrap();
        Ok(Database {
            client,
            name_space: String::from("surreal"),
            db_name: String::from("users"),
        })
    }

    pub async fn get_all_users(&self) -> Option<Vec<Users>> {
        let result = self.client.select("users").await;
        match result {
            Ok(all_users) => Some(all_users),
            Err(_) => None,
        }
    }
}
