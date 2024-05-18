use sqlx::{Pool, MySql, Error, MySqlPool};
use crate::model::types;

use super::types::Project;


pub async fn connect() -> Result <Pool<MySql>, Error> {
    MySqlPool::connect("mysql://WifiVisualizerUser@localhost:3306/WifiViewer").await
}

pub async fn get_internal_user_id(oauth_user_id: &str) -> Option<types::User> {
    let connection = connect().await;

    match connection {
        Err(err) => {
            eprintln!("[ERROR]Cannot connect to Database [{}]", err.to_string());
            None
        },
        Ok(pool) => {
            dbg!(&oauth_user_id);
            let query: types::User = 
                sqlx::query_as("SELECT * FROM Users WHERE oauth_user_id = ?")
                    .bind(&oauth_user_id)
                    .fetch_one(&pool)
                    .await
                    .ok()?;
            dbg!(&query);
            Some(query)
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
                        sqlx::query_as("SELECT * FROM AuthProviders WHERE provider_name = ?")
                            .bind(oauth_provider)
                            .fetch_one(&pool)
                            .await?;


            sqlx::query("INSERT INTO Users(OAuth_provider_id, OAuth_user_id) VALUES (?, ?)")
                .bind(provider.get_provider_id())
                .bind(oauth_user_id)
                .execute(&pool)
                .await?;

            Ok(())
        }
    }
}

pub async fn get_or_attempt_insert_user_id(oauth_user_id: &str, oauth_provider: &str) -> Option<types::User> {
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

pub async fn get_project_list(user: types::User) -> Option<Vec<types::Project>> {
    let pool = connect().await.ok()?;

    let project_list : Vec<Project> = 
        sqlx::query_as("SELECT * FROM Projects WHERE creator_user_id = ?")
        .bind(user.get_internal_id())
        .fetch_all(&pool)
        .await
        .ok()?;
    
    Some(project_list)
}
