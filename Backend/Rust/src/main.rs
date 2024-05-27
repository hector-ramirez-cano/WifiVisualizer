mod tests;
mod internal;
mod controller;
mod model;

extern crate serial;

#[macro_use] extern crate rocket;

// std imports
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// crate imports
use rocket::fs::{FileServer, relative};
use rocket_oauth2::OAuth2;

// own crate imports
pub use crate::internal::frame_type::*;
use crate::internal::frame_ops::*;
use crate::internal::procs::*;
use crate::internal::logger::Logger;
use crate::internal::threading_comm::Message;

#[launch]
fn launch() -> _ {
    println!("[DEBUG]Launching API Server");
    let fileserver = FileServer::from(relative!("../../Frontend/public/"));
    let logger = Arc::new(Mutex::new(Logger::new()));
    let (tx_web, rx_web) = mpsc::channel::<Message>();
    let (tx_esp, rx_esp) = mpsc::channel::<Message>();

    let rocket = rocket::build()
        .mount("/public", fileserver)
        .mount("/", routes![
            controller::web::index,
            controller::web::view,
            controller::web::home,
            controller::web::capture,
            controller::web::login,
            controller::web::logout,
            controller::web::invalid_msg,

            controller::auth::google_login,
            controller::auth::google_auth_callback,

            controller::api::get_project_list,
            controller::api::get_connection_status,
            controller::api::get_terminal_contents,
            controller::api::post_capture_request,
        ])
        .manage(logger.clone())
        .manage((tx_web, Mutex::new(rx_esp)))
        .attach(OAuth2::<controller::auth::Google>::fairing("google"));

    thread::spawn( move || {
        controller::esp32_backend::launch_esp32_backend(logger, rx_web, tx_esp);
    } );
    

    rocket
}


