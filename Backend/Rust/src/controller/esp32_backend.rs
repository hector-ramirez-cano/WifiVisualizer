// std imports
use std::collections::HashMap;
use std::{thread, time::Duration};
use std::sync::{mpsc, Arc, Mutex};

use serial::unix::TTYPort;

// own imports
use crate::{proc_rx_logs, proc_rx_request_ack, proc_tx_handshake, proc_tx_reset, rx_frame_blocking, Cmd, FrameStack, NetworkId, Position, BSSID};
use crate::internal::threading_comm::Message;
use crate::internal::logger::{Logger, Severity};
use crate::internal::frame_type::*;
use crate::create_port_conn;
use crate::model::{self, db};


type ThreadReceiver = mpsc::Receiver<Message>;
type ThreadSender   = mpsc::Sender<Message>;
fn handle_thread_msg(logger : &Arc<Mutex<Logger>>, rx_thread: &ThreadReceiver, tx_thread: &ThreadSender, port_status: bool) -> Option<Message> {
    let msg = if let Ok(msg) = rx_thread.try_recv() {
        msg
    } else {
        return None
    };

    match msg {
        Message::StartCapture(_)      => {},
        Message::BackendReady(_)      => panic!("Unreachable!"),
        Message::BackendStatusRequest => {
            println!("[INFO][LOCAL]handling request for backend status");
            let mut msg = tx_thread.send(Message::BackendReady(true));
            while let Err(e) = msg {
                if let Ok(mut handle) = logger.lock() {
                    handle.log(Severity::ERROR, &format!("Failed to transmit backend status with error '{}'. Retrying in 50ms", e.to_string()));
                    thread::sleep(Duration::from_millis(50));
                }
                msg = tx_thread.send(Message::BackendReady(port_status));
            }
        },
    }

    Some(msg)
}

fn terminate_esp32_backend(logger : Arc<Mutex<Logger>>, tx_thread: ThreadSender) {
    // Inform Rocket the backend is no longer ready
    while let Err(e) = tx_thread.send(Message::BackendReady(false)) {
        if let Ok(mut handle) = logger.lock() {
            handle.log(Severity::ERROR, &format!("Failed to transmit backend status with error '{}'. Retrying in 50ms", e.to_string()));
            thread::sleep(Duration::from_millis(50));
        }
    }
}

fn capture_project_data(logger : &Arc<Mutex<Logger>>, rx_thread: &ThreadReceiver, tx_thread: &ThreadSender, project: model::types::Project, frame_stack: &mut FrameStack, conn: &mut TTYPort) {
    let mut ssids       : HashMap<NetworkId, SSID       > = HashMap::new();
    let mut bssids      : HashMap<NetworkId, BSSID      > = HashMap::new();
    let mut rssi_records: HashMap<Position , Vec<Record>> = HashMap::new();

    loop {
        // If any messages comes from the web thread, handle them now
        handle_thread_msg(&logger, &rx_thread, &tx_thread, true);

        // Rx a frame or log the error and loop back
        let frame = match rx_frame_blocking(frame_stack,conn) {
            Ok(frame) => frame,
            Err(e) => {
                    if let Ok(mut handle) = logger.lock() {
                        handle.log(Severity::ERROR, &format!("Failed to receive frame with error '{:?}'. Retrying in 50ms", e));
                        thread::sleep(Duration::from_millis(50));
                    }
                    continue;
                }
        };
        
        // Act depending of the frame type
        match frame.get_cmd() {
            Cmd::EndOfTransmission => break,
            Cmd::AddBSSID     { id, bssid } => { bssids.insert(id.clone(), bssid.clone()); },
            Cmd::AddSSID      { id, ssid   } => { ssids.insert (id.clone(), ssid.clone() ); }
            Cmd::RequestAck   { frame_id: _  } => proc_rx_request_ack(conn, frame_stack, logger.clone()).unwrap(),
            Cmd::TransmitLogs { logs } => { proc_rx_logs(&mut logger.clone(), &logs); },
            Cmd::RecordRSSI   { position, record_count: _, records } => { 
                // add records to tally
                // if key doesn't exist, create with records, otherwise, append it to running record
                rssi_records
                    .entry(position.clone())
                    .and_modify(|position_vec| position_vec.append(&mut records.clone()))
                    .or_insert(records.clone());
            }
            _ => {}
        }
    }

    


}

fn acquire_port(logger : &Arc<Mutex<Logger>>, rx_thread: &ThreadReceiver, tx_thread: &ThreadSender, port_name: &str) -> TTYPort {
    loop {
        let port = create_port_conn(&port_name);

        // if any status requests come, state the backend is not ready
        handle_thread_msg(&logger, &rx_thread, &tx_thread, false);

        // either unwrap the port, or log the error and try again in 2 secs
        match port {
            Ok(port) => return port,
            Err(err) => {
                if let Ok(mut handle) = logger.lock() {
                    handle.log(Severity::ERROR, &format!("Failed to bind to port {} with error {}. Retrying in 2s", &err, &port_name));
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    };
}

fn await_capture_order(logger : &Arc<Mutex<Logger>>, rx_thread: &ThreadReceiver, tx_thread: &ThreadSender) -> model::types::Project {
    loop {
        // Process status requests
        let msg = handle_thread_msg(&logger, &rx_thread, &tx_thread, false);    

        if let Some(msg) = msg {
            match msg {
                Message::StartCapture(project) => return project,
                _ => {}
            }
        }
    }
}

pub fn launch_esp32_backend(logger : Arc<Mutex<Logger>>, rx_thread: ThreadReceiver, tx_thread: ThreadSender) { 
    let config = crate::internal::config::load_config().unwrap_or_default();

    let port_name = config.esp32_port();

    // Try to acquire handle for the port
    let mut conn = acquire_port(&logger, &rx_thread, &tx_thread, port_name);

    // we've acquired the handle to the port, log it.
    if let Ok(mut handle) = logger.lock() {
        handle.log(Severity::INFO, &format!("Acquired connection to port {}", &port_name));
    }
    
    // TODO: Remove assert in favor of error handling
    // Perform handshake with ESP32, we're ready to start the transmission
    let mut frame_stack = FrameStack::new();
    assert_eq!(Ok(()), proc_tx_handshake(&mut conn, &mut frame_stack, logger.clone()));

    // log it lul
    if let Ok(mut handle) = logger.lock() {
        handle.log(Severity::INFO, "Sucessful handshake with ESP32");
    }
    

    // wait for the order to start the capture. Comes asyncronously from the web thread
    let user = await_capture_order(&logger, &rx_thread, &tx_thread);

    // Perform the reset of the connection. After its completion, the ESP32 will begin capture
    assert_eq!(Ok(()), proc_tx_reset    (&mut conn, &mut frame_stack));

    capture_project_data(&logger, &rx_thread, &tx_thread, user, &mut frame_stack, &mut conn);

    terminate_esp32_backend(logger, tx_thread);
}