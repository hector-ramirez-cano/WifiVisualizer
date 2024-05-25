#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
use rocket::serde::json;

#[cfg(test)]
use serial::SerialPort;

#[cfg(test)]
use crate::internal::frame_ops::*;

#[cfg(test)]
use crate::internal::procs;

#[cfg(test)]
use crate::FrameError;


#[cfg(test)]
use crate::FrameStack;

#[cfg(test)]
use crate::internal::frame_type::{Checksum, Frame, Position, StepSize, BSSID, Cmd, NetworkId, Record, RSSI, SSID};

#[test]
fn test_parse_header() {
    let bytes: [u8; 5] = [0x10, 0x00, 0x33, 0x44, 0x55];
    assert_eq!(Frame::parse_header(&bytes), Err(FrameError::NotEnoughBytes));


    let bytes: [u8; 8] = [0x10, 0x00, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
    assert_eq!(Frame::parse_header(&bytes), Ok((0x01_u8, 0x00, 0x08_u16, 0x33445566_u32)));


    let bytes: [u8; 10] = [0x70, 0x02, 0x33, 0x44, 0x55, 0x66, 0xFF, 0xFF, 0x77, 0x88];
    assert_eq!(Frame::parse_header(&bytes), Ok((0x07_u8, 0x02_u16, 0x0A_u16, 0x33445566_u32)));
}

#[test]
fn test_frame_parse_sot() {
    let bytes: [u8; 1] = [0x00];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xDB, 0xC1];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::StartOfTransmission, 1).unwrap() , 8)));

    let bytes: [u8; 10] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xDB, 0xC1, 0x54, 0xF3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::StartOfTransmission, 1).unwrap() , 8)));
}

#[test]
fn test_frame_parse_ack() {
    let bytes: [u8; 1] = [0x40];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 12] = [0x40, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x11, 0x18];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Ack {frame_id: 5}, 10).unwrap(), 12)));

    let bytes: [u8; 14] = [0x40, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x11, 0x18, 0x54, 0xF3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Ack {frame_id: 5}, 10).unwrap(), 12)));

    let bytes: [u8; 16] = [0x40, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x11, 0x18, 0x8E, 0xD6, 0x54, 0xF3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Ack {frame_id: 5}, 10).unwrap(), 12)));
}

#[test]
fn test_frame_parse_reset() {
    let bytes: [u8; 1] = [0x10];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 12] = [0x10, 0x01, 0x00, 0x00, 0x00, 0x01, 0x7B, 0xFB, 0x00, 0x00, 0x00, 0x01];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    let bytes: [u8; 8] = [0x10, 0x00, 0x00, 0x00, 0x00, 0x01, 0x4B, 0xC3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Reset, 1).unwrap(), 8)));

    let bytes: [u8; 12] = [0x10, 0x00, 0x00, 0x00, 0x00, 0x01, 0x4B, 0xC3, 0x00, 0x00, 0x00, 0x01];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Reset, 1).unwrap(), 8)));

    let bytes: [u8; 14] = [0x10, 0x00, 0x00, 0x00, 0x00, 0x01, 0x4B, 0xC3, 0x00, 0x00, 0x00, 0x01, 0x54, 0xF3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Reset, 1).unwrap(), 8)));
}

#[test]
fn test_frame_parse_ready() {
    let bytes: [u8; 1] = [0x20];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 8] = [0x20, 0x00, 0x00, 0x00, 0x00, 0x01, 0xBB, 0xC6];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Ready, 1).unwrap(), 8)));

    let bytes: [u8; 10] = [0x20, 0x00, 0x00, 0x00, 0x00, 0x01, 0xBB, 0xC6, 0x3F, 0xD0];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Ready, 1).unwrap(), 8)));

    let bytes: [u8; 12] = [0x20, 0x00, 0x00, 0x00, 0x00, 0x01, 0xBB, 0xC6, 0x3F, 0xD0, 0x54, 0xF3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Ready, 1).unwrap(), 8)));

}

#[test]
fn test_frame_parse_request_pos() {
    let bytes: [u8; 1] = [0x30];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 8] = [0x30, 0x00, 0x00, 0x00, 0x00, 0x01 , 0x2B, 0xC4];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::RequestPosition, 1).unwrap(), 8)));

    let bytes: [u8; 10] = [0x30, 0x00, 0x00, 0x00, 0x00, 0x01 , 0x2B, 0xC4, 0x54, 0xF3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::RequestPosition, 1).unwrap(), 8)));

}

#[test]
fn test_frame_parse_request_retransmit() {
    let bytes: [u8; 1] = [0x50];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 16] = [0x50, 0x08, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0xBA, 0x96];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::RequestRetransmit {frame_id_start: 5, frame_id_end: 5}, 10).unwrap(), 16)));

    let bytes: [u8; 18] = [0x50, 0x08, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0xBA, 0x96, 0x54, 0xF3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::RequestRetransmit {frame_id_start: 5, frame_id_end: 5}, 10).unwrap(), 16)));
}

#[test]
fn test_frame_parse_request_ack() {
    let bytes: [u8; 1] = [0x60];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 12] = [0x60, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x7B, 0x19];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::RequestAck {frame_id: 5}, 10).unwrap(), 12)));

    let bytes: [u8; 14] = [0x60, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x7B, 0x19, 0x54, 0xF3];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::RequestAck {frame_id: 5}, 10).unwrap(), 12)));
}


#[test]
fn test_frame_parse_length_check() {
    // Length = 10, not enough bytes in buffer
    let bytes : [u8; 4]= [0x70, 0x0A, 0x00, 0x00]; 
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));
}

#[test]
fn test_frame_parse_add_bssid() {
    
    // no length
    let bytes: [u8; 1] = [0x80];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 6] = [0x80, 0x00, 0x00, 0x00, 0x00, 0x01];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    let bytes: [u8; 6] = [0x80, 0x03, 0x00, 0x00, 0x00, 0x01];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 9] = [0x80, 0x03, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0xFF];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 37, actual length >= 37
    let bytes: [u8; 48] = [0x80, 0x25, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00 ];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 64, actual length >= 64
    let bytes: [u8; 78] = [0x80, 0x40, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x0 ];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 4, no data
    let bytes: [u8; 6] = [0x80, 0x04, 0x00, 0x00, 0x00, 0x01]; 
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    let bytes: [u8; 18] = [0x80, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0xF0, 0x1F];
     // length = 4, Id = 1
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::AddBSSID { id: NetworkId::from_int(1), bssid: BSSID::from_bytes(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]).unwrap() }, 1).unwrap() , 18)));
}

#[test]
fn test_frame_parse_add_ssid() {
    // no length
    let bytes: [u8; 1] = [0x70]; 
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    // length = 0
    let bytes: [u8; 6] = [0x70, 0x00, 0x00, 0x00, 0x00, 0x01]; 
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 3
    let bytes: [u8; 6] = [0x70, 0x03, 0x00, 0x00, 0x00, 0x01]; 
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    // length = 3
    let bytes: [u8; 9] = [0x70, 0x03, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0xFF]; 
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 37, actual length >= 37
    let bytes: [u8; 48] = [0x70, 0x25, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x0 ];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 64, actual length >= 64
    let bytes: [u8; 78] = [0x70, 0x40, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x00,0x00, 0x00, 0x00, 0x0 ];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 4, no data
    let bytes: [u8; 6] = [0x70, 0x04, 0x00, 0x00, 0x00, 0x01]; 
    assert_eq!(Frame::parse(&bytes), Err(FrameError::NotEnoughBytes));

    // length = 4, Id = 1
    let bytes: [u8; 12] = [0x70, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0xEC, 0xBC]; 
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(1), ssid: SSID::from_str("").unwrap() }, 1).unwrap() , 12)));

    // length = 4, Id = 5
    let bytes: [u8; 12] = [0x70, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x2F, 0xBD]; 
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(5), ssid: SSID::from_str("").unwrap() }, 1).unwrap() , 12)));

    // length = 4, Id = 251 988 481
    let bytes: [u8; 12] = [0x70, 0x04, 0x00, 0x00, 0x00, 0x01, 0x0F, 0x05, 0x0A, 0x01, 0x59, 0xA9]; 
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(251_988_481), ssid: SSID::from_str("").unwrap() }, 1).unwrap() , 12)));

    // length = 5, Id = 1
    let bytes: [u8; 13] = [0x70, 0x05, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x41, 0x44, 0x7C]; 
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(1), ssid: SSID::from_str("A").unwrap() }, 1).unwrap() , 13)));

    // Notice if more bytes are given, they're ignored
    // length = 5, Id = 1
    let bytes: [u8; 16] = [0x70, 0x05, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x41, 0x44, 0x7C, 0xD0, 0xC5, 0x42 ]; 
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(1), ssid: SSID::from_str("A").unwrap() }, 1).unwrap() , 13)));
    
    // length = 6, Id = 1
    let bytes: [u8; 14] = [0x70, 0x06, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x41, 0x42, 0x94, 0xCA]; 
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(1), ssid: SSID::from_str("AB").unwrap() }, 1).unwrap() , 14)));

    let bytes: [u8; 44] = [0x70, 0x24, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0xD7, 0x6E];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(1), ssid: SSID::from_str("ABABABABABABABABABABABABABABABAB").unwrap()}, 1).unwrap(), 44)));

    let bytes: [u8; 43] = [0x70, 0x25, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x42];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

}

#[test]
fn test_frame_parse_record_rssi() {
    let bytes: [u8; 10] = [0x90, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    let bytes: [u8; 18] = [0x90, 0x0B, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 0x11, pitch = 1, yaw = 2, record count = 0
    let bytes: [u8; 18] = [0x90, 0x0C, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    // length = 0x11, pitch = FF, yaw = DD, record count = 1, internal id = EE, RSSI = -128
    let bytes: [u8; 23] = [0x90, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xDD, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xEE, 0b1000_0000];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::RSSIValueOutOfRange));

    // length = 0x11, pitch = 2, yaw = 1, record count = 1, internal id = 1, RSSI = 1
    let bytes: [u8; 23] = [0x90, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::RSSIValueOutOfRange));

    // length = 0x11, pitch = 2, yaw = 1, record count = 1, internal id = 1, RSSI = -82
    let bytes: [u8; 25] = [0x90, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0b1010_1110, 0x5A, 0x5F];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::RecordRSSI { position: crate::Position::from_int(1, 2), record_count: 1, records: vec![Record::from_components(NetworkId::from_int(1), RSSI::from_int(-82).unwrap())] }, 1).unwrap(), 25)));
}

#[test]
fn test_frame_parse_set_position() {
    let bytes: [u8; 14] = [0xA0, 0x07, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02];
    assert_eq!(Frame::parse(&bytes), Err(FrameError::LengthValueOutOfRange));

    let bytes: [u8; 16] = [0xA0, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x78, 0xA5];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::SetPosition { position: Position::from_int(1, 2) }, 1).unwrap(), 16)));
}


#[test]
fn test_frame_parse_set_param() {
    // pitch = 1, Yaw = 2, step = 0xFF, masurements_per_step = 5
    let bytes: [u8; 25] = [0xB0, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x05, 0xF5, 0x0F];
    let left = Frame::parse(&bytes);
    let right = Ok((Frame::from_cmd(Cmd::SetParams { position: Position::from_int(1, 2), step_size: StepSize::from_pitch_yaw(0x11223344, 0x55667788).unwrap(), measurements_per_step: 5 }, 1).unwrap(), 25));
    assert_eq!(left, right);
}

#[test]
fn test_frame_parse() {
    // pitch = 1, Yaw = 2, step = 0xFF, masurements_per_step = 5
    let bytes: [u8; 25] = [0xB0, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x05, 0xF5, 0x0F];
    assert_eq!(
        Frame::parse(&bytes), 
        Ok(
            (
                Frame::from_components(
                    Cmd::SetParams { position: Position::from_int(1, 2), step_size: StepSize::from_pitch_yaw(0x11223344, 0x55667788).unwrap(), measurements_per_step: 5 },
                    1,
                    Checksum::from_int(0xF50F)
                ).unwrap(),
                25
            )
        )
    )

}


#[test]
fn test_frame_to_bytes() {
    // Start of transmission
    let bytes: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xDB, 0xC1];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::StartOfTransmission, 1).unwrap().as_bytes().unwrap());

    // Reset
    let bytes: [u8; 8] = [0x10, 0x00, 0x00, 0x00, 0x00, 0x01, 0x4B, 0xC3];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::Reset, 1).unwrap().as_bytes().unwrap());

    // Ready
    let bytes: [u8; 8] = [0x20, 0x00, 0x00, 0x00, 0x00, 0x01, 0xBB, 0xC6];
    assert_eq!(Frame::parse(&bytes), Ok((Frame::from_cmd(Cmd::Ready, 1).unwrap(), 8)));

    // RequestPosition
    let bytes: [u8; 8] = [0x30, 0x00, 0x00, 0x00, 0x00, 0x01 , 0x2B, 0xC4];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::RequestPosition, 1).unwrap().as_bytes().unwrap());

    // Ack
    let bytes: [u8; 12] = [0x40, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x11, 0x18];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::Ack {frame_id: 5}, 10).unwrap().as_bytes().unwrap());

    // RequestRetransmit
    let bytes: [u8; 16] = [0x50, 0x08, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0xBA, 0x96];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::RequestRetransmit {frame_id_start: 5, frame_id_end: 5}, 10).unwrap().as_bytes().unwrap());

    // RequestAck
    let bytes: [u8; 12] = [0x60, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x7B, 0x19];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::RequestAck {frame_id: 5}, 10).unwrap().as_bytes().unwrap());


    // AddSSID
    let bytes: [u8; 44] = [0x70, 0x24, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0xD7, 0x6E];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(1), ssid: SSID::from_str("ABABABABABABABABABABABABABABABAB").unwrap() }, 1).unwrap().as_bytes().unwrap());

    // AddBSSID
    let bytes: [u8; 18] = [0x80, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0xF0, 0x1F];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::AddBSSID { id: NetworkId::from_int(1), bssid: BSSID::from_bytes(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]).unwrap() }, 1).unwrap().as_bytes().unwrap());

    // RecordRSSI
    let bytes: [u8; 25] = [0x90, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0b1010_1110, 0x5A, 0x5F];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::RecordRSSI { position: crate::Position::from_int(1, 2), record_count: 1, records: vec![Record::from_components(NetworkId::from_int(1), RSSI::from_int(-82).unwrap())] }, 1).unwrap().as_bytes().unwrap());

    // SetPosition
    let bytes: [u8; 16] = [0xA0, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x78, 0xA5];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::SetPosition { position: Position::from_int(1, 2) }, 1).unwrap().as_bytes().unwrap());

    // SetParam
    let bytes: [u8; 25] = [0xB0, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x05, 0xF5, 0x0F];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::SetParams { position: Position::from_int(1, 2), step_size: StepSize::from_pitch_yaw(0x11223344, 0x55667788).unwrap(), measurements_per_step: 5 }, 1).unwrap().as_bytes().unwrap());

    // TransmitPicture
    let bytes : [u8; 18] = [0xC0, 0x0A, 0xDD, 0xDD, 0xDD, 0xDD, 0xFA, 0x00, 0x00, 0xAF, 0xC1, 0x00, 0x00, 0x1C, 0x7B, 0x7D, 0x8B, 0x60];
    let frame = Frame::from_cmd(Cmd::TransmitPicture { position: Position::from_int(0xFA0000AF, 0xC100001C), body: json::json! ({}) }, 0xDDDDDDDD).unwrap();
    assert_eq!(bytes.to_vec(), frame.as_bytes().unwrap());

    // EndOfTransmission
    let bytes: [u8; 8] = [0xF0, 0x00, 0x00, 0x00, 0x00, 0x01, 0x2B, 0xD5];
    assert_eq!(bytes.to_vec(), Frame::from_cmd(Cmd::EndOfTransmission, 1).unwrap().as_bytes().unwrap());
}


#[test]
fn test_frame_ping() {
    // Ping the messages to the ESP32, by encoding -->  decoding -> encoding --> decoding
    //                                    └───PC────┴───────── ESP32 ─────────┴─── PC ───┘
    // Tests the whole loop, both for Rust and Python

    use crate::internal::frame_ops::{create_port_conn, rx_frame_blocking};
    let mut conn = create_port_conn("/dev/ttyUSB0").unwrap();

    
    fn ping_frame<T: SerialPort>(port: &mut T, frame: &crate::Frame) -> Vec<u8> {

        let mut frame_stack = FrameStack::new();
        port.write_all(&frame.as_bytes().unwrap()).unwrap();
        rx_frame_blocking(&mut frame_stack, port).unwrap().as_bytes().unwrap()
    }

    // StartOfTransmission
    let bytes: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xDB, 0xC1];
    let frame = Frame::from_cmd(Cmd::StartOfTransmission, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // Reset
    let bytes: [u8; 8] = [0x10, 0x00, 0x00, 0x00, 0x00, 0x01, 0x4B, 0xC3];
    let frame = Frame::from_cmd(Cmd::Reset, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // Ready
    let bytes: [u8; 8] = [0x20, 0x00, 0x00, 0x00, 0x00, 0x01, 0xBB, 0xC6];
    let frame = Frame::from_cmd(Cmd::Ready, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // RequestPosition
    let bytes: [u8; 8] = [0x30, 0x00, 0x00, 0x00, 0x00, 0x01 , 0x2B, 0xC4];
    let frame = Frame::from_cmd(Cmd::RequestPosition, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);
    
    // Ack
    let bytes: [u8; 12] = [0x40, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x11, 0x18];
    let frame = Frame::from_cmd(Cmd::Ack {frame_id: 5}, 10).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);
    
    // RequestRetransmit
    let bytes: [u8; 16] = [0x50, 0x08, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0xBA, 0x96];
    let frame = Frame::from_cmd(Cmd::RequestRetransmit {frame_id_start: 5, frame_id_end: 5}, 10).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // RequestAck
    let bytes: [u8; 12] = [0x60, 0x04, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x05, 0x7B, 0x19];
    let frame = Frame::from_cmd(Cmd::RequestAck {frame_id: 5}, 10).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // AddSSID
    let bytes: [u8; 44] = [0x70, 0x24, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0x41, 0x42, 0xD7, 0x6E];
    let frame = Frame::from_cmd(Cmd::AddSSID { id: NetworkId::from_int(1), ssid: SSID::from_str("ABABABABABABABABABABABABABABABAB").unwrap() }, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // AddBSSID
    let bytes: [u8; 18] = [0x80, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0xF0, 0x1F];
    let frame = Frame::from_cmd(Cmd::AddBSSID { id: NetworkId::from_int(1), bssid: BSSID::from_bytes(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]).unwrap() }, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // RecordRSSI
    let bytes: [u8; 25] = [0x90, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0b1010_1110, 0x5A, 0x5F];
    let frame = Frame::from_cmd(Cmd::RecordRSSI { position: crate::Position::from_int(1, 2), record_count: 1, records: vec![Record::from_components(NetworkId::from_int(1), RSSI::from_int(-82).unwrap())] }, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // SetPosition
    let bytes: [u8; 16] = [0xA0, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x78, 0xA5];
    let frame = Frame::from_cmd(Cmd::SetPosition { position: Position::from_int(1, 2) }, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // SetParam
    let bytes: [u8; 25] = [0xB0, 0x11, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x05, 0xF5, 0x0F];
    let frame = Frame::from_cmd(Cmd::SetParams { position: Position::from_int(1, 2), step_size: StepSize::from_pitch_yaw(0x11223344, 0x55667788).unwrap(), measurements_per_step: 5 }, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // TransmitPicture
    let bytes : [u8; 18] = [0xC0, 0x0A, 0xDD, 0xDD, 0xDD, 0xDD, 0xFA, 0x00, 0x00, 0xAF, 0xC1, 0x00, 0x00, 0x1C, 0x7B, 0x7D, 0x8B, 0x60];
    let frame = Frame::from_cmd(Cmd::TransmitPicture { position: Position::from_int(0xFA0000AF, 0xC100001C), body: json::json! ({}) }, 0xDDDDDDDD).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // TransmitLogs
    let bytes : [u8; 15] = [0xD0, 0x07, 0xDD, 0xDD, 0xDD, 0xDD, 0x7B, 0x22, 0x41, 0x22, 0x3A, 0x30, 0x7D, 0xC3, 0xC9 ];
    let frame = Frame::from_cmd(Cmd::TransmitLogs { logs: json::json!({"A":0}) }, 0xDDDDDDDD).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);

    // EndOfTransmission
    let bytes: [u8; 8] = [0xF0, 0x00, 0x00, 0x00, 0x00, 0x01, 0x2B, 0xD5];
    let frame = Frame::from_cmd(Cmd::EndOfTransmission, 1).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);
    
}

/*#[test]
fn test_ping_jumbo_frames() {
    // Ping the messages to the ESP32, by encoding -->  decoding -> encoding --> decoding
    //                                    └───PC────┴───────── ESP32 ─────────┴─── PC ───┘
    // Tests the whole loop, both for Rust and Python

    use crate::internal::frame_ops::{create_port_conn, rx_frame_blocking};
    let mut conn = create_port_conn("/dev/ttyUSB0").unwrap();

    
    fn ping_frame<T: SerialPort>(port: &mut T, frame: &crate::Frame) -> Vec<u8> {

        let mut frame_stack = FrameStack::new();
        port.write_all(&frame.as_bytes().unwrap()).unwrap();
        rx_frame_blocking(&mut frame_stack, port).unwrap().as_bytes().unwrap()
    }

    // TransmitLogs
    let bytes : [u8; 1736] = [
        0xD6, 0xC6,
        0xDD, 0xDD, 0xDD, 0xDD,
        0x7b, 0x22, 0x6c, 0x6f, 0x67, 0x73, 0x22, 0x3a, 0x5b, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x53, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x20, 0x49, 0x6e, 0x74, 0x65, 0x66, 0x61, 0x63, 0x65, 0x20, 0x69, 0x6e, 0x69, 0x74, 0x69, 0x61, 0x6c, 0x69, 0x7a, 0x65, 0x64, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x33, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x57, 0x61, 0x69, 0x74, 0x69, 0x6e, 0x67, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x68, 0x61, 0x6e, 0x64, 0x73, 0x68, 0x61, 0x6b, 0x65, 0x2e, 0x2e, 0x2e, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x33, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x48, 0x61, 0x6e, 0x64, 0x73, 0x68, 0x61, 0x6b, 0x65, 0x20, 0x70, 0x65, 0x72, 0x66, 0x6f, 0x72, 0x6d, 0x65, 0x64, 0x20, 0x73, 0x75, 0x63, 0x63, 0x65, 0x73, 0x73, 0x66, 0x75, 0x6c, 0x6c, 0x79, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x31, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x5f, 0x73, 0x74, 0x65, 0x70, 0x3d, 0x39, 0x35, 0x34, 0x34, 0x33, 0x37, 0x31, 0x38, 0x34, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x31, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x79, 0x61, 0x77, 0x5f, 0x73, 0x74, 0x65, 0x70, 0x3d, 0x32, 0x31, 0x34, 0x37, 0x34, 0x38, 0x33, 0x36, 0x34, 0x38, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x31, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x5f, 0x73, 0x74, 0x65, 0x70, 0x3d, 0x39, 0x35, 0x34, 0x34, 0x33, 0x37, 0x31, 0x38, 0x34, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x31, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x79, 0x61, 0x77, 0x5f, 0x73, 0x74, 0x65, 0x70, 0x3d, 0x32, 0x31, 0x34, 0x37, 0x34, 0x38, 0x33, 0x36, 0x34, 0x38, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x6f, 0x6e, 0x50, 0x6f, 0x73, 0x69, 0x74, 0x69, 0x6f, 0x6e, 0x43, 0x68, 0x61, 0x6e, 0x67, 0x65, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x33, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x4d, 0x65, 0x61, 0x73, 0x75, 0x72, 0x69, 0x6e, 0x67, 0x20, 0x63, 0x75, 0x72, 0x72, 0x65, 0x6e, 0x74, 0x20, 0x70, 0x69, 0x74, 0x63, 0x68, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x79, 0x61, 0x77, 0x2e, 0x2e, 0x2e, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x34, 0x30, 0x30, 0x35, 0x34, 0x33, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x32, 0x36, 0x2e, 0x36, 0x35, 0x32, 0x34, 0x34, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x34, 0x2e, 0x39, 0x31, 0x36, 0x36, 0x36, 0x37, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x34, 0x39, 0x36, 0x33, 0x31, 0x31, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x35, 0x33, 0x2e, 0x39, 0x33, 0x32, 0x34, 0x36, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x39, 0x2e, 0x38, 0x33, 0x33, 0x33, 0x33, 0x33, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x33, 0x36, 0x32, 0x32, 0x33, 0x36, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x38, 0x30, 0x2e, 0x33, 0x33, 0x34, 0x38, 0x34, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x31, 0x34, 0x2e, 0x37, 0x35, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x33, 0x37, 0x34, 0x32, 0x30, 0x37, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x31, 0x30, 0x36, 0x2e, 0x38, 0x31, 0x35, 0x33, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x31, 0x39, 0x2e, 0x36, 0x36, 0x36, 0x36, 0x37, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x34, 0x38, 0x36, 0x37, 0x33, 0x34, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x31, 0x33, 0x34, 0x2e, 0x30, 0x33, 0x32, 0x34, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x32, 0x34, 0x2e, 0x35, 0x38, 0x33, 0x33, 0x33, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x33, 0x37, 0x36, 0x36, 0x30, 0x31, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x31, 0x36, 0x30, 0x2e, 0x35, 0x32, 0x38, 0x35, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x32, 0x39, 0x2e, 0x35, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x34, 0x36, 0x30, 0x33, 0x39, 0x38, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x31, 0x38, 0x37, 0x2e, 0x35, 0x37, 0x32, 0x38, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x33, 0x34, 0x2e, 0x34, 0x31, 0x36, 0x36, 0x36, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x34, 0x31, 0x30, 0x31, 0x32, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x32, 0x31, 0x34, 0x2e, 0x32, 0x38, 0x37, 0x38, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x33, 0x39, 0x2e, 0x33, 0x33, 0x33, 0x33, 0x33, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x34, 0x38, 0x39, 0x31, 0x32, 0x38, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x32, 0x34, 0x31, 0x2e, 0x35, 0x32, 0x30, 0x36, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x34, 0x34, 0x2e, 0x32, 0x35, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x61, 0x63, 0x63, 0x5f, 0x78, 0x3d, 0x34, 0x2e, 0x34, 0x34, 0x31, 0x32, 0x34, 0x34, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x70, 0x69, 0x74, 0x63, 0x68, 0x3d, 0x32, 0x36, 0x38, 0x2e, 0x34, 0x33, 0x39, 0x34, 0x2c, 0x20, 0x79, 0x61, 0x77, 0x3d, 0x34, 0x39, 0x2e, 0x31, 0x36, 0x36, 0x36, 0x37, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x33, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x6d, 0x65, 0x61, 0x73, 0x75, 0x72, 0x65, 0x64, 0x20, 0x70, 0x69, 0x74, 0x63, 0x68, 0x20, 0x3d, 0x20, 0x32, 0x36, 0x2e, 0x38, 0x34, 0x33, 0x39, 0x34, 0xb0, 0x2c, 0x20, 0x72, 0x65, 0x71, 0x75, 0x65, 0x75, 0x65, 0x73, 0x74, 0x65, 0x64, 0x20, 0x70, 0x69, 0x74, 0x63, 0x68, 0x20, 0x3d, 0x20, 0x31, 0x30, 0x2e, 0x30, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x33, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x6d, 0x65, 0x61, 0x73, 0x75, 0x72, 0x65, 0x64, 0x20, 0x79, 0x61, 0x77, 0x20, 0x20, 0x20, 0x3d, 0x20, 0x34, 0x2e, 0x39, 0x31, 0x36, 0x36, 0x36, 0x37, 0xb0, 0x2c, 0x20, 0x72, 0x65, 0x71, 0x75, 0x65, 0x75, 0x65, 0x73, 0x74, 0x65, 0x64, 0x20, 0x79, 0x61, 0x77, 0x20, 0x3d, 0x20, 0x30, 0x2e, 0x30, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x33, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x4d, 0x6f, 0x76, 0x69, 0x6e, 0x67, 0x20, 0x76, 0x65, 0x72, 0x74, 0x69, 0x63, 0x61, 0x6c, 0x6c, 0x79, 0x20, 0x2d, 0x31, 0x36, 0x2e, 0x38, 0x34, 0x33, 0x39, 0x34, 0xb0, 0x2c, 0x20, 0x72, 0x61, 0x74, 0x69, 0x6f, 0x3d, 0x38, 0x2e, 0x30, 0x2c, 0x20, 0x74, 0x6f, 0x74, 0x61, 0x6c, 0x3d, 0x2d, 0x31, 0x33, 0x34, 0x2e, 0x37, 0x35, 0x31, 0x35, 0xb0, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x33, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x4d, 0x6f, 0x76, 0x69, 0x6e, 0x67, 0x20, 0x68, 0x6f, 0x72, 0x69, 0x7a, 0x6f, 0x6e, 0x74, 0x61, 0x6c, 0x6c, 0x79, 0x20, 0x2d, 0x34, 0x2e, 0x39, 0x31, 0x36, 0x36, 0x36, 0x37, 0xb0, 0x2c, 0x20, 0x72, 0x61, 0x74, 0x69, 0x6f, 0x3d, 0x38, 0x2e, 0x30, 0x2c, 0x20, 0x74, 0x6f, 0x74, 0x61, 0x6c, 0x3d, 0x2d, 0x33, 0x39, 0x2e, 0x33, 0x33, 0x33, 0x33, 0x34, 0xb0, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x33, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x52, 0x65, 0x73, 0x65, 0x74, 0x20, 0x70, 0x65, 0x72, 0x66, 0x6f, 0x72, 0x6d, 0x65, 0x64, 0x20, 0x73, 0x75, 0x63, 0x63, 0x65, 0x73, 0x73, 0x66, 0x75, 0x6c, 0x6c, 0x79, 0x22, 0x7d, 0x2c, 0x7b, 0x22, 0x73, 0x65, 0x76, 0x65, 0x72, 0x69, 0x74, 0x79, 0x22, 0x3a, 0x31, 0x2c, 0x22, 0x6d, 0x73, 0x67, 0x22, 0x3a, 0x22, 0x54, 0x72, 0x61, 0x6e, 0x73, 0x6d, 0x69, 0x74, 0x20, 0x6c, 0x6f, 0x67, 0x73, 0x22, 0x7d, 0x5d, 0x7d,
        0xEC, 0x03 ];
    let frame = Frame::from_cmd(Cmd::TransmitLogs { logs: json::json!({"logs":[{"severity":2,"msg":"Serial Inteface initialized"},{"severity":3,"msg":"Waiting for handshake..."},{"severity":3,"msg":"Handshake performed successfully"},{"severity":1,"msg":"pitch_step=954437184"},{"severity":1,"msg":"yaw_step=2147483648"},{"severity":1,"msg":"pitch_step=954437184"},{"severity":1,"msg":"yaw_step=2147483648"},{"severity":2,"msg":"onPositionChange"},{"severity":3,"msg":"Measuring current pitch and yaw..."},{"severity":2,"msg":"acc_x=4.400543"},{"severity":2,"msg":"pitch=26.65244, yaw=4.916667"},{"severity":2,"msg":"acc_x=4.496311"},{"severity":2,"msg":"pitch=53.93246, yaw=9.833333"},{"severity":2,"msg":"acc_x=4.362236"},{"severity":2,"msg":"pitch=80.33484, yaw=14.75"},{"severity":2,"msg":"acc_x=4.374207"},{"severity":2,"msg":"pitch=106.8153, yaw=19.66667"},{"severity":2,"msg":"acc_x=4.486734"},{"severity":2,"msg":"pitch=134.0324, yaw=24.58333"},{"severity":2,"msg":"acc_x=4.376601"},{"severity":2,"msg":"pitch=160.5285, yaw=29.5"},{"severity":2,"msg":"acc_x=4.460398"},{"severity":2,"msg":"pitch=187.5728, yaw=34.41666"},{"severity":2,"msg":"acc_x=4.41012"},{"severity":2,"msg":"pitch=214.2878, yaw=39.33333"},{"severity":2,"msg":"acc_x=4.489128"},{"severity":2,"msg":"pitch=241.5206, yaw=44.25"},{"severity":2,"msg":"acc_x=4.441244"},{"severity":2,"msg":"pitch=268.4394, yaw=49.16667"},{"severity":3,"msg":"measured pitch = 26.84394°, requeuested pitch = 10.0"},{"severity":3,"msg":"measured yaw   = 4.916667°, requeuested yaw = 0.0"},{"severity":3,"msg":"Moving vertically -16.84394°, ratio=8.0, total=-134.7515°"},{"severity":3,"msg":"Moving horizontally -4.916667°, ratio=8.0, total=-39.33334°"},{"severity":3,"msg":"Reset performed successfully"},{"severity":1,"msg":"Transmit logs"}]}) }, 0xDDDDDDDD).unwrap();
    let rx = ping_frame(&mut conn, &frame);
    assert_eq!(bytes.to_vec(), rx);
}*/

/*
#[test]
fn test_transmission_loop() {
    use crate::frame_ops::*;
    use crate::procs::*;

    let mut conn = create_port_conn("/dev/ttyUSB0").unwrap();

    let mut frame_stack = FrameStack::new();

    assert_eq!(Ok(()), proc_tx_handshake(&mut conn, &mut frame_stack));
    assert_eq!(Ok(()), proc_tx_reset    (&mut conn, &mut frame_stack));

    loop {
        let frame = frame_ops::rx_frame_blocking(&mut frame_stack,&mut conn).unwrap();
        println!("{frame:?}");

        match frame.get_cmd() {
            Cmd::EndOfTransmission => break,
            Cmd::RequestAck { frame_id } => procs::proc_rx_request_ack(&mut conn, frame, &mut frame_stack).unwrap(),
            other => {}
        }
    }
}
*/