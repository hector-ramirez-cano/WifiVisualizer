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
use crate::internal::logger::{Severity, Logger};
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
        launch_esp32_backend(logger, rx_web, tx_esp)
    } );
    

    rocket
}


// TODO: Move to controller layer
type ThreadReceiver = mpsc::Receiver<Message>;
type ThreadSender   = mpsc::Sender<Message>;
fn launch_esp32_backend(logger : Arc<Mutex<Logger>>, rx_thread: ThreadReceiver, tx_thread: ThreadSender) { 
    let config = crate::internal::config::load_config().unwrap_or_default();

    let port_name = config.esp32_port();
    let mut conn = 'port: loop {
        let port = create_port_conn(&port_name);

        match port {
            Ok(port) => break 'port port,
            Err(err) => {
                    if let Ok(mut handle) = logger.lock() {
                        handle.log(Severity::ERROR, &format!("Failed to bind to port {} with error {}. Retrying in 5s", &err, &port_name));
                        thread::sleep(Duration::from_secs(5));

                        // Best effort. If it can't be sent, try again
                        let _ = tx_thread.send(Message::BackendReady(false));
                    }
                }
        }
    };

    if let Ok(mut handle) = logger.lock() {
        handle.log(Severity::INFO, &format!("Acquired connection to port {}", &port_name));
    }

    let mut frame_stack = FrameStack::new();

    // TODO: Remove assert in favor of error handling
    assert_eq!(Ok(()), proc_tx_handshake(&mut conn, &mut frame_stack, logger.clone()));

    if let Ok(mut handle) = logger.lock() {
        handle.log(Severity::INFO, "Sucessful handshake with ESP32");
    }
    // Inform Rocket the backend is ready
    let mut msg = tx_thread.send(Message::BackendReady(true));
    while let Err(e) = msg {
        if let Ok(mut handle) = logger.lock() {
            handle.log(Severity::ERROR, &format!("Failed to transmit backend status with error '{}'. Retrying in 50ms", e.to_string()));
            thread::sleep(Duration::from_millis(50));
        }
        msg = tx_thread.send(Message::BackendReady(true));
    }

    

    // wait for the order to start the capture
    loop {
        // Process status requests
        if let Ok(msg) = rx_thread.try_recv() {
            match msg {
                Message::StartCapture => todo!(),
                Message::BackendReady(_) => todo!(),
                Message::BackendStatusRequest => {
                    println!("[INFO][LOCAL]handling request for backend status");
                    let mut msg = tx_thread.send(Message::BackendReady(true));
                    while let Err(e) = msg {
                        if let Ok(mut handle) = logger.lock() {
                            handle.log(Severity::ERROR, &format!("Failed to transmit backend status with error '{}'. Retrying in 50ms", e.to_string()));
                            thread::sleep(Duration::from_millis(50));
                        }
                        msg = tx_thread.send(Message::BackendReady(true));
                    }
                },
            }
        }

    }

    assert_eq!(Ok(()), proc_tx_reset    (&mut conn, &mut frame_stack));
    loop {
        let frame = rx_frame_blocking(&mut frame_stack,&mut conn).unwrap();
        println!("{frame:?}");

        match frame.get_cmd() {
            Cmd::EndOfTransmission => break,
            Cmd::RequestAck   { frame_id: _ } => proc_rx_request_ack(&mut conn, &mut frame_stack, logger.clone()).unwrap(),
            Cmd::TransmitLogs { logs } => { proc_rx_logs(&mut logger.clone(), &logs); },
            _ => {}
        }
    }

    // Inform Rocket the backend is no longer ready
    while let Err(e) = tx_thread.send(Message::BackendReady(false)) {
        if let Ok(mut handle) = logger.lock() {
            handle.log(Severity::ERROR, &format!("Failed to transmit backend status with error '{}'. Retrying in 50ms", e.to_string()));
            thread::sleep(Duration::from_millis(50));
        }
    }
}