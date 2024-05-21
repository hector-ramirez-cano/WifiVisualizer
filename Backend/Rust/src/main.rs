mod tests;
mod internal;
mod controller;
mod model;

extern crate serial;

#[macro_use] extern crate rocket;


use rocket::fs::{FileServer, relative};
use rocket_oauth2::OAuth2;

pub use crate::internal::frame_type::*;
use crate::internal::frame_ops::*;
use crate::internal::procs::*;

#[launch]
fn launch() -> _ {
    println!("[DEBUG]Launching API Server");
    let fileserver = FileServer::from(relative!("../../Frontend/public/"));

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
    .attach(OAuth2::<controller::auth::Google>::fairing("google"))
}

fn foo() { 

    loop {
        
    }

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