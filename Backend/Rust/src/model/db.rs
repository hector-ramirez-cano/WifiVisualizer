use std::collections::HashMap;

use rocket::serde::json;
use sqlx::{Pool, MySql, Error, MySqlPool};
use crate::{internal, model::types};

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

pub async fn new_project(user: types::User, title: String, description: String) -> Result<types::Project, sqlx::Error> {
    let connection = connect().await;

    match connection {
        Err(err) => {
            eprintln!("[ERROR]Cannot connect to Database [{}]", err.to_string());
            Err(err)
        },
        Ok(pool) => {
            sqlx::query("INSERT INTO Projects(project_title, project_description, in_capture, creator_user_id, image_id, project_data) VALUES (?, ?, ?, ?, ?, ?);")
                .bind(title)
                .bind(description)
                .bind(true)
                .bind(user.get_internal_id())
                .bind(1)  // TODO: Update with actual image id
                .bind("{}")
                .execute(&pool)
                .await?;

            let project : types::Project = 
                sqlx::query_as("SELECT * FROM Projects WHERE creator_user_id = ? ORDER BY project_id LIMIT 1")
                .bind(user.get_internal_id())
                .fetch_one(&pool)
                .await?;
        

            Ok(project)
        }
    }
}

type ProjectRecords = HashMap<internal::frame_type::Position , Vec<internal::frame_type::Record>>;
type ProjectSSIDs   = HashMap<internal::frame_type::NetworkId, Vec<internal::frame_type::SSID  >>;
type ProjectBSSIDs  = HashMap<internal::frame_type::NetworkId, Vec<internal::frame_type::BSSID >>;
pub async fn update_project(project: types::Project, contents: json::Value) -> Result<(), sqlx::Error> {
    let connection = connect().await;

    match connection {
        Err(err) => {
            eprintln!("[ERROR]Cannot connect to Database [{}]", err.to_string());
            Err(err)
        },
        Ok(pool) => {
            sqlx::query("UPDATE Projects SET project_data = ? WHERE project_id = ?")
            .bind(contents)
            .bind(project.project_id())
            .execute(&pool)
            .await?;

            Ok(())
        },
        
    }
}