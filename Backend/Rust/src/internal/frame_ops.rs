
// std crates
use std::{io, thread};

// External crates
use serial::prelude::*;
use serial::unix::TTYPort;
use std::time::Duration;

// own crates
pub use crate::internal::frame_type::*;


pub fn create_port_conn(port_name: &str) -> io::Result<TTYPort> {
    let mut port = serial::open(port_name)?;
    if let Err(e) = port.set_timeout(Duration::from_secs(25)) {
        dbg!("[DEBUG]Failed to set timeout with error ", e);
    }

    port.reconfigure(&|settings| {
        settings.set_baud_rate   (serial::Baud115200)?;
        settings.set_char_size   (serial::Bits8);
        settings.set_parity      (serial::ParityNone);
        settings.set_stop_bits   (serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;

    Ok(port)
}

pub fn rx_frame<T: SerialPort>(frame_stack : &mut FrameStack, port: &mut T) -> Result<Frame, FrameError> {
    let mut header: [u8; FRAME_HEADER_SIZE] = [0, 0, 0, 0, 0, 0];

    match port.read_exact(&mut header) {
        Ok (_) => {},
        Err(_) => {return Err(FrameError::TransmissionTimedOut);}
    }

    // We parse the header to see how many bytes we need to read
    let (_, length, frame_length, _) = Frame::parse_header(&header)?;

    // We read the rest, and to avoid unnecesary copies, we also read the Checksum. We're gonna need it anyways
    let mut buff = vec![0; length as usize + CHECKSUM_SIZE];

    
    match port.read_exact(&mut buff) {
        Ok(_) => {
            let mut complete_buff = Vec::with_capacity(frame_length as usize);
            complete_buff.append(&mut header.to_vec());
            complete_buff.append(&mut buff);

            let frame = Frame::parse(&complete_buff)?.0;
            frame_stack.append_rx_frame(frame.clone());
            Ok(frame)
        },
        Err(e) => {
            print!("[DEBUG]Failed to read frame body with error {}", e);
            panic!("[DEBUG]Failed to read frame body with error {}", e);
        },
    }

}

pub fn rx_frame_blocking<T: SerialPort>(frame_stack : &mut FrameStack, port: &mut T) -> Result<Frame, FrameError> {

    loop {
        match rx_frame(frame_stack, port) {
            Ok (frame)  => return Ok(frame),
            Err(e) => {
                println!("[INFO ]Failed to read with error '{e:?}', retrying in 500ms...");
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

pub fn rx_frame_blocking_expect<T: SerialPort>(frame_stack : &mut FrameStack, port: &mut T, expected_cmd_code : u8) -> Result<Frame, FrameError> {
    
    let frame = rx_frame_blocking(frame_stack, port)?;

    if frame.get_cmd().as_int()? != expected_cmd_code {
        return Err(FrameError::InvalidCommandSequence)
    }
    
    Ok(frame)
    
}

pub fn tx_new_frame<T: SerialPort>(cmd: Cmd, frame_stack : &mut FrameStack, port: &mut T) -> Result<(), FrameError> {
    let frame = Frame::from_cmd(cmd, frame_stack.curr_id())?;
    tx_frame_blocking(frame, frame_stack, port)
}

pub fn tx_frame_blocking<T: SerialPort>(frame: Frame, frame_stack : &mut FrameStack, port: &mut T) -> Result<(), FrameError> {
    let bytes = frame.as_bytes()?;
    match port.write_all(&bytes) {
        Ok(_) => {
            frame_stack.append_tx_frame(frame);
            Ok(())
        },
        Err(_) => Err(FrameError::FailedToTransmitFrame),
    }
}

pub fn retx_frame_blocking<T: SerialPort>(frame: Frame, port: &mut T) -> Result<(), FrameError> {
    let bytes = frame.as_bytes()?;
    match port.write_all(&bytes) {
        Ok(_) => {
            Ok(())
        },
        Err(_) => Err(FrameError::FailedToTransmitFrame),
    }
}