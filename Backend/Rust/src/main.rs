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
        controller::api::index,
        controller::api::view,
        controller::api::home,
        controller::api::capture,
        controller::api::login,
        controller::api::logout,
        controller::api::invalid_msg,

        controller::api::google_login,
        controller::api::google_auth_callback,

        controller::api::api_get_project_list,

        ])
    .attach(OAuth2::<controller::api::Google>::fairing("google"))
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