// std imports
use std::{thread, time::Duration};
use std::sync::{mpsc, Arc, Mutex};

// own imports
use crate::{proc_rx_logs, proc_rx_request_ack, proc_tx_handshake, proc_tx_reset, rx_frame_blocking, Cmd, FrameStack};
use crate::internal::threading_comm::Message;
use crate::internal::logger::{Logger, Severity};
use crate::create_port_conn;


type ThreadReceiver = mpsc::Receiver<Message>;
type ThreadSender   = mpsc::Sender<Message>;
fn handle_thread_msg(logger : &Arc<Mutex<Logger>>, rx_thread: &ThreadReceiver, tx_thread: &ThreadSender, port_status: bool) -> Option<Message> {
    if let Ok(msg) = rx_thread.try_recv() {
        match msg {
            Message::StartCapture         => todo!(),
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
    }

    None
}

pub fn launch_esp32_backend(logger : Arc<Mutex<Logger>>, rx_thread: ThreadReceiver, tx_thread: ThreadSender) { 
    let config = crate::internal::config::load_config().unwrap_or_default();

    let port_name = config.esp32_port();
    let mut conn = 'port: loop {
        let port = create_port_conn(&port_name);

        handle_thread_msg(&logger, &rx_thread, &tx_thread, false);

        match port {
            Ok(port) => break 'port port,
            Err(err) => {
                if let Ok(mut handle) = logger.lock() {
                    handle.log(Severity::ERROR, &format!("Failed to bind to port {} with error {}. Retrying in 2s", &err, &port_name));
                    thread::sleep(Duration::from_secs(2));
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
    

    // wait for the order to start the capture
    loop {
        // Process status requests
        let msg = handle_thread_msg(&logger, &rx_thread, &tx_thread, false);    

        if let Some(msg) = msg {
            match msg {
                Message::StartCapture => break,
                _ => {}
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