
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::{http::CookieJar, serde::json::Value};

use crate::model::db;


pub const OAUTH2_TOKEN_COOKIE : & 'static str = "oauth_token";
pub const OAUTH2_USER_ID      : & 'static str = "oauth_user_id";


#[get("/api/<user_id>/project_list")]
pub async fn get_project_list(user_id: &str, cookies : &CookieJar<'_>) -> Value {

    let cookie_id = cookies.get(&OAUTH2_USER_ID);
    let invalid = if let Some(val) = cookie_id {
        // name=value
        val.to_string().split("=").collect::<Vec<_>>()[1] != user_id
    } else { 
        true //cookie was none
    };

    if invalid {
        return rocket::serde::json::json!(
            {
                "code": 403,
                "comment": "Provided user id does not match logged in user id",
                "list": []
            }
        )
    }

    
    let internal_user = db::get_or_attempt_insert_user_id(user_id, "google").await;
    let user = match internal_user {
        None => {
            return rocket::serde::json::json!(
                {
                    "code": 500,
                    "comment": "Could not create new user id",
                    "list": []
                }
            );
        }

        Some(user) => user
    };

    dbg!(&user);
    dbg!(db::get_project_list(user).await);
    

    // TODO: Consult db
    rocket::serde::json::json!(
        {
            "code": 200,
            "list": [
                {
                    "name": "Capture #1",
                    "description": "Wassup baby",
                    "status": "done",
                    "date": "01/02/03"
                },
                {
                    "name": "Capture #2",
                    "description": "How you livin'",
                    "status": "done",
                    "date": "04/05/06"
                },
                {
                    "name": "Capture #2",
                    "description": "Oh, you haven't seen how mean this dean can be- ean",
                    "status": "done",
                    "date": "07/08/09"
                },
                {
                    "name": "Noice",
                    "description": "I forgot everything you said after rectum!",
                    "status": "in capture",
                    "date": "10/11/12"
                }
            ]
        }
    )
}

#[get("/api/connection_status")]
pub async fn get_connection_status() -> Value {
    rocket::serde::json::json! (
        {
            "status": {
                "esp32_cam": {
                    "up": true,
                    "ready": true
                },
                "esp32": {
                    "up": true,
                    "ready": true
                },
                "backend": {
                    "up": true,
                    "ready": true
                }
            }
        }
    )
}

#[derive(FromForm, Debug)]
pub struct CaptureRequest {
    #[field(validate = range(1..=180))]
    step_x_deg: u32,

    #[field(validate = range(1..=20))]
    step_y_deg: u32,

    #[field(validate = range(1..=20))]
    measurements_per_step: u8
}

#[post("/api/start", data = "<params>")]
pub fn post_capture_request(params: Form<CaptureRequest>) -> () {
    println!("{:?}", params);
}