mod tests;
mod internal;
mod controller;
mod model;

extern crate serial;

#[macro_use] extern crate rocket;


use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use rocket::fs::{FileServer, relative};
use rocket_oauth2::OAuth2;

pub use crate::internal::frame_type::*;
use crate::internal::frame_ops::*;
use crate::internal::procs::*;
use crate::internal::logger::{log, Severity, Logger};
use crate::internal::threading_comm::Message;

#[launch]
fn launch() -> _ {
    println!("[DEBUG]Launching API Server");
    let fileserver = FileServer::from(relative!("../../Frontend/public/"));
    let mut logger = Arc::new(Mutex::new(Logger::new()));
    let (tx_web, rx_web) = mpsc::channel::<Message>();
    let (tx_esp, rx_esp) = mpsc::channel::<Message>();

    log(&mut logger, Severity::DEBUG  , "Mensaje de ejemplo 1");
    log(&mut logger, Severity::INFO   , "Mensaje de ejemplo 2");
    log(&mut logger, Severity::ERROR  , "Mensaje de ejemplo 3");
    log(&mut logger, Severity::WARNING, "Mensaje de ejemplo 4");
    log(&mut logger, Severity::VERBOSE, "Mensaje de ejemplo 5");
    log(&mut logger, Severity::DEBUG  , "Mensaje de ejemplo 6");
    log(&mut logger, Severity::DEBUG  , "Mensaje de ejemplo 7");

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
        launch_esp32_backend(logger, rx_web, tx_esp)
    } );
    

    rocket
}


type ThreadReceiver = mpsc::Receiver<Message>;
type ThreadSender   = mpsc::Sender<Message>;
fn launch_esp32_backend(logger : Arc<Mutex<Logger>>, rx_thread: ThreadReceiver, tx_thread: ThreadSender) { 
    let port_name = "/dev/ttyUSB0";
    let mut conn = 'port: loop {
        let port = create_port_conn(&port_name);

        if let Ok(port) = port {
            break 'port port;
        } else {
            if let Ok(mut handle) = logger.lock() {
                handle.log(Severity::ERROR, &format!("Failed to bind to port {}. Retrying in 500ms", &port_name));
                thread::sleep(Duration::from_millis(500));

                // Best efford. If it can't be sent, try again
                let _ = tx_thread.send(Message::BackendReady(false));
            }
        }
    };

    let mut frame_stack = FrameStack::new();

    // TODO: Remove assert in favor of error handling
    assert_eq!(Ok(()), proc_tx_handshake(&mut conn, &mut frame_stack));
    assert_eq!(Ok(()), proc_tx_reset    (&mut conn, &mut frame_stack));

    // Inform Rocket the backend is ready
    while let Err(e) = tx_thread.send(Message::BackendReady(true)) {
        if let Ok(mut handle) = logger.lock() {
            handle.log(Severity::ERROR, &format!("Failed to transmit backend status with error '{}'. Retrying in 50ms", e.to_string()));
            thread::sleep(Duration::from_millis(50));
        }
    }

    loop {
        let frame = rx_frame_blocking(&mut frame_stack,&mut conn).unwrap();
        println!("{frame:?}");

        match frame.get_cmd() {
            Cmd::EndOfTransmission => break,
            Cmd::RequestAck   { frame_id: _ } => proc_rx_request_ack(&mut conn, &mut frame_stack).unwrap(),
            Cmd::TransmitLogs { logs } => { proc_rx_logs(&mut logger.clone(), &logs); },
            _ => {}
        }
    }
}