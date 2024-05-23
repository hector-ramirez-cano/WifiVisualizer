mod tests;
mod internal;
mod controller;
mod model;

extern crate serial;

#[macro_use] extern crate rocket;


use std::sync::{Arc, Mutex};

use rocket::fs::{FileServer, relative};
use rocket_oauth2::OAuth2;

pub use crate::internal::frame_type::*;
use crate::internal::frame_ops::*;
use crate::internal::procs::*;
use crate::internal::logger::{log, Severity, Logger};

#[launch]
fn launch() -> _ {
    println!("[DEBUG]Launching API Server");
    let fileserver = FileServer::from(relative!("../../Frontend/public/"));

    let mut logger = Arc::new(Mutex::new(Logger::new()));

    log(&mut logger, Severity::DEBUG  , "Mensaje de ejemplo 1");
    log(&mut logger, Severity::INFO   , "Mensaje de ejemplo 2");
    log(&mut logger, Severity::ERROR  , "Mensaje de ejemplo 3");
    log(&mut logger, Severity::WARNING, "Mensaje de ejemplo 4");
    log(&mut logger, Severity::VERBOSE, "Mensaje de ejemplo 5");
    log(&mut logger, Severity::DEBUG  , "Mensaje de ejemplo 6");
    log(&mut logger, Severity::DEBUG  , "Mensaje de ejemplo 7");

    let rocket =
    rocket::build()
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
    .attach(OAuth2::<controller::auth::Google>::fairing("google"));

    // foo(logger.clone());

    rocket
}

fn foo(logger : Arc<Mutex<Logger>>) { 

    let mut conn = create_port_conn("/dev/ttyUSB0").unwrap();

    let mut frame_stack = FrameStack::new();

    assert_eq!(Ok(()), proc_tx_handshake(&mut conn, &mut frame_stack));
    assert_eq!(Ok(()), proc_tx_reset    (&mut conn, &mut frame_stack));

    loop {
        let frame = rx_frame_blocking(&mut frame_stack,&mut conn).unwrap();
        println!("{frame:?}");

        match frame.get_cmd() {
            Cmd::EndOfTransmission => break,
            Cmd::RequestAck { frame_id: _ } => proc_rx_request_ack(&mut conn, &mut frame_stack).unwrap(),
            _ => {}
        }
    }
}