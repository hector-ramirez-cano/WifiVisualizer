mod tests;
mod internal;
mod controller;

extern crate serial;

#[macro_use] extern crate rocket;


use rocket::fs::{FileServer, relative};

pub use crate::internal::frame_type::*;
use crate::internal::frame_ops::*;
use crate::internal::procs::*;

#[launch]
fn launch() -> _ {
    println!("[DEBUG]Launching API Server");
    let fileserver = FileServer::from(relative!("../../../Frontend/public/"));
    dbg!(&fileserver);

    rocket::build()
    .mount("/public", fileserver)
    .mount("/", routes![
        controller::api::index,
        controller::api::login,
        ])
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