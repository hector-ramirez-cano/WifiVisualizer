// std crates
use std::sync::{Arc, Mutex};

// External crates
use serial::prelude::*;
use rocket::serde::json;

// own crates
use crate::internal::frame_ops::{self, tx_frame_blocking};
use crate::internal::logger::{Severity, Logger};
use crate::internal::frame_type::*;

pub fn proc_tx_reset<T: SerialPort>(port: &mut T, frame_stack: &mut FrameStack) -> Result<(), FrameError> {
    
    // tx SetParams
    let frame = Frame::from_cmd(
        Cmd::SetParams {
            position: Position::from_degrees(10f32, 0f32)?,
            step_size: StepSize::from_degrees(80f32, 180f32)?,
            measurements_per_step: 1
        }, 
        frame_stack.curr_id()
    )?;
    frame_ops::tx_frame_blocking(frame, frame_stack, port)?;

    // rx Ack
    let _ = frame_ops::rx_frame_blocking_expect(frame_stack, port, Cmd::Ack { frame_id: 1 }.as_int()?)?;

    // rx Ready
    let _ = frame_ops::rx_frame_blocking_expect(frame_stack, port, Cmd::Ready.as_int()?)?;

    // tx ready
    let frame = Frame::from_cmd(Cmd::Ready, frame_stack.curr_id())?;
    tx_frame_blocking(frame, frame_stack, port)?;

    Ok(())
}


pub fn proc_tx_handshake<T: SerialPort>(port: &mut T, frame_stack: &mut FrameStack, logger : Arc<Mutex<Logger>>) -> Result<(), FrameError> {
    let mut handshake_frame_stack = FrameStack::new();

    // tx SoT
    let sot = Frame::from_cmd(Cmd::StartOfTransmission, 0)?;
    frame_ops::tx_frame_blocking(sot, &mut handshake_frame_stack, port)?;

    // rx Ack
    while frame_ops::rx_frame(frame_stack, port).is_err() {
        // Retransmit until we get a response
        if let Ok(mut handle) = logger.lock() {
            handle.log(Severity::ERROR, "Failed to perform handshake; no answer. Trying again...");
        }
        let sot = Frame::from_cmd(Cmd::StartOfTransmission, 0)?;
        frame_ops::retx_frame_blocking(sot, port)?;
    }

    // tx Reset
    let frame = Frame::from_cmd(Cmd::Reset, 1)?;
    frame_ops::tx_frame_blocking(frame, &mut handshake_frame_stack, port)?;

    // rx Ack
    let _ = frame_ops::rx_frame_blocking_expect(frame_stack, port, Cmd::Ack { frame_id:1 }.as_int()?);

    Ok(())
}

pub fn proc_tx_request_retransmit<T: SerialPort>(port: &mut T, frame_stack: &mut FrameStack, frame_id_start: u32, frame_id_end: u32) -> Result<(), FrameError> {
    frame_ops::tx_new_frame(Cmd::RequestRetransmit { frame_id_start, frame_id_end }, frame_stack, port)
}


pub fn proc_tx_ack<T: SerialPort>(port: &mut T, frame_stack: &mut FrameStack, frame_id: u32) -> Result<(), FrameError> {
    frame_ops::tx_new_frame(Cmd::Ack { frame_id }, frame_stack, port)
}


pub fn proc_rx_request_ack<T: SerialPort>(port: &mut T, frame_stack: &mut FrameStack, logger : Arc<Mutex<Logger>>) -> Result<(), FrameError> {
    let rx_queue = frame_stack.get_rx_frame_queue();
    let mut rx_ids: Vec<u32> = Vec::with_capacity(rx_queue.len());

    // store only the IDs
    for (id, _) in rx_queue {
        rx_ids.push(*id);
    }

    // sort low -> high
    rx_ids.sort();

    let most_recent_ack = frame_stack.get_remote_ackd_frame_id();
    let mut new_most_recent_ack = most_recent_ack;
    for id in rx_ids {
        if id < new_most_recent_ack {
            continue;
        }

        if id == new_most_recent_ack {
            new_most_recent_ack += 1;
            continue;
        }

        // we've lost some packets. We ask for them again and exit
        let start = new_most_recent_ack;
        let end = id;
        if let Ok(mut handle) = logger.lock() {
            handle.log(Severity::DEBUG, &format!("Lost frames! [{} - {}]", start, end));
        }
        proc_tx_request_retransmit(port, frame_stack, start, end)?;
        return Ok(());
        

    }

    // if we got here, it means we can safely ack all ids up until new_most_recent_ack, as they're contiguous
    proc_tx_ack(port, frame_stack, new_most_recent_ack)?;
    frame_stack.set_remote_ackd_frame_id(new_most_recent_ack);

    Ok(())
}

pub fn proc_rx_logs_process_log(logger : &mut std::sync::MutexGuard<Logger>, log: &json::Value) -> Option<()> {
    let log = log.as_object()?;
    
    let severity =  Severity::try_from( log.get("severity")?.as_number()?.as_u64()? ).ok()?;
    let msg = log.get("msg")?.as_str()?;

    logger.log(severity, msg);


    Some(())
    
}

pub fn proc_rx_logs (logger : &mut Arc<Mutex<Logger>>, logs: &json::Value) -> Option<()> {
    let logs = match logs {
        json::Value::Object(root) => root,
        _ => { return None; /* Ignore, Early bail */}
    };

    let logs = logs.get("logs")?.as_array()?;
    
    if let Ok(mut logger) = logger.lock() {
        for log in logs {
            proc_rx_logs_process_log(&mut logger, log);
        }
    }
    
    Some(())
}