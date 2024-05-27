use std::clone;

use serde::Serialize;


#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct User {
    user_id          : i64,
    oauth_user_id    : String,
    oauth_provider_id: i64
}

impl User {
    pub fn get_internal_id(&self) -> i64 { return self.user_id; }
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)] 
pub struct AuthProvider {
    provider_id  : i64,
    provider_name: String
}

impl AuthProvider {
    pub fn get_provider_id(&self) -> i64 { return self.provider_id; }
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Project {
    project_id          : i64,
    project_title       : String,
    project_description : String,
    in_capture          : bool,
    creator_user_id     : i64,
    image_id            : i64,
    project_data        : sqlx::types::JsonValue
}

impl Project {
    pub fn project_id(&self) -> i64 {
        self.project_id
    }
    
    pub fn creator_user_id(&self) -> i64 {
        self.creator_user_id
    }
    
    pub fn image_id(&self) -> i64 {
        self.image_id
    }
}