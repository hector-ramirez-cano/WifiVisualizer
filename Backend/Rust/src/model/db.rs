use sqlx::{Pool, MySql, Error, MySqlPool};
use crate::model::types;


pub async fn connect() -> Result <Pool<MySql>, Error> {
    MySqlPool::connect("mysql://WifiVisualizerUser@localhost:3306/WifiViewer").await
}

pub async fn get_internal_user_id(oauth_user_id: &str) -> Option<i64> {
    let connection = connect().await;

    match connection {
        Err(err) => {
            eprintln!("[ERROR]Cannot connect to Database [{}]", err.to_string());
            None
        },
        Ok(pool) => {
            let query: types::User = 
                sqlx::query_as("SELECT * FROM Users WHERE oauth_user_id = $1")
                    .bind(oauth_user_id)
                    .fetch_one(&pool)
                    .await
                    .ok()?;
            dbg!(&query);
            Some(query.get_internal_id())
        }
    }
}

pub async fn insert_user_id(oauth_user_id: &str, oauth_provider: &str) -> Result<(), Error> {
    let connection = connect().await;

    match connection {
        Err(err) => {
            eprintln!("[ERROR]Cannot connect to Database [{}]", err.to_string());
            Err(err)
        },
        Ok(pool) => {
            let provider : types::AuthProvider = 
                        sqlx::query_as("SELECT provider_id FROM AuthProviders WHERE provider_name = $1")
                            .bind(oauth_provider)
                            .fetch_one(&pool)
                            .await?;


            sqlx::query("INSERT INTO Users VALUES ($1, $2)")
                .bind(oauth_user_id)
                .bind(provider.get_provider_id())
                .execute(&pool)
                .await?;

            Ok(())
        }
    }
}


pub async fn get_or_attempt_insert_user_id(oauth_user_id: &str, oauth_provider: &str) -> Option<i64> {
    let user_id = get_internal_user_id(oauth_user_id).await;

    match user_id {
        None => {
            // Attempt to insert it
            match insert_user_id(oauth_user_id, oauth_provider).await {
                Ok(_) => get_internal_user_id(oauth_user_id).await,
                Err(_) => None
            }
        },
        Some(_) => user_id
    }
}
