use std::process::ExitStatus;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};

use rocket::form::Form;
use rocket::State;
use rocket::{http::CookieJar, serde::json};

use crate::internal::logger::Logger;
use crate::internal::threading_comm::Message;
use crate::model::db;


pub const OAUTH2_TOKEN_COOKIE : & 'static str = "oauth_token";
pub const OAUTH2_USER_ID      : & 'static str = "oauth_user_id";


#[get("/api/<user_id>/project_list", rank=10)]
pub async fn get_project_list(user_id: &str, cookies : &CookieJar<'_>) -> json::Value {

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
                    "name": "Test #1",
                    "description": "I forgot everything you said after rectum!",
                    "status": "in capture",
                    "date": "10/11/12"
                }
            ]
        }
    )
}

type ThreadReceiver = mpsc::Receiver<Message>;
type ThreadSender   = mpsc::Sender<Message>;
#[get("/api/connection_status")]
pub async fn get_connection_status(threading_comm : &State<(ThreadSender, Mutex<ThreadReceiver>)>) -> json::Value {
    // let (tx_web, rx_esp) = (threading_comm.0, threading_comm.1);

    let backend_ready = {
        // inquiry about the the status of the esp32 backend
        if let Ok(receiver) = threading_comm.1.lock() {
            Ok(Message::BackendReady(true)) == receiver.try_recv()
        } else {
            false    
        }
    };

    let config = crate::internal::config::load_config().unwrap_or_default();

    let esp32_cam_up = {
        let ip = config.esp32_cam_ip();
        if let Ok(ping) = Command::new("ping")
            .args(["-c 1", "-w 2", &ip.to_canonical().to_string()])
            .output() {
                ExitStatus::success(&ping.status)
            } else {
                false
            }
    };

    rocket::serde::json::json! (
        {
            "status": {
                "esp32_cam": {
                    "up": esp32_cam_up,
                    "ready": true
                },
                "esp32": {
                    "up": true, // always true, since the backend runs on the same program as the backend web
                    "ready": backend_ready
                },
                "backend": {
                    "up": true,
                    "ready": true
                }
            }
        }
    )
}

#[get("/api/terminal/<start>", rank=1)]
pub async fn get_terminal_contents(start: usize, logger:  &State<Arc<Mutex<Logger>>>) -> json::Value {
    if let Ok(handle) = logger.lock() {
        if handle.get_logs().len() > start{
            let lines = &handle.get_logs().as_slice()[start..];

            rocket::serde::json::json! {{
                "code": 200,
                "lines": lines
            }}
        } else {
            rocket::serde::json::json! {{
                "code": 200,
                "lines": []
            }}
        }

        
    } else {
        rocket::serde::json::json! {{
            "code": 500,
            "lines": []
        }}
    }
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