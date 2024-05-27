use std::str::{from_utf8, FromStr};
use std::result::Result;
use std::collections::VecDeque;
use rocket::serde::json;
use serde::Serialize;
use crate::internal::utils::*;

extern crate rocket;

static CRC_16 : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_MODBUS);
pub const FRAME_HEADER_SIZE: usize = 6;
pub const CHECKSUM_SIZE: usize = 2;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct BSSID {
    bytes: [u8; 6]
}


#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct SSID {
    name: String
}


#[derive(PartialEq, Debug, Clone)]
pub struct Checksum {
    checksum: u16
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct RSSI {
    strength: i8
}

#[derive(PartialEq, Debug, Clone, Hash, Eq, Serialize)]
pub struct Position {
    pitch: u32,
    yaw: u32,
}

#[derive(PartialEq, Debug, Clone, Hash, Eq, Serialize)]
pub struct NetworkId {
    id: u32
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Record {
    internal_id: NetworkId,
    rssi       : RSSI
}

#[derive(PartialEq, Debug, Clone)]
pub struct StepSize {
    pitch_step: u32,
    yaw_step  : u32
}


#[derive(PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum Cmd {
    StartOfTransmission = 0,
    Reset            = 1,
    Ready            = 2,
    RequestPosition  = 3,
    Ack              {frame_id: u32                                                       } = 4,
    RequestRetransmit{frame_id_start: u32, frame_id_end: u32                              } = 5,
    RequestAck       {frame_id: u32                                                       } = 6,
    AddSSID          {id: NetworkId, ssid: SSID                                           } = 7,
    AddBSSID         {id: NetworkId, bssid : BSSID                                        } = 8,
    RecordRSSI       {position: Position , record_count: u32 , records: Vec<Record>       } = 9,
    SetPosition      {position: Position                                                  } = 10,
    SetParams        {position: Position , step_size : StepSize, measurements_per_step: u8} = 11,
    TransmitPicture  {position: Position , body      : json::Value                        } = 12,
    TransmitLogs     {logs    : json::Value                                               } = 13,

    EndOfTransmission = 15
}

#[derive(PartialEq, Debug, Clone)]
pub struct Frame {
    cmd: Cmd,
    frame_id: u32,
    checksum: Checksum
}


#[derive(PartialEq, Debug, Clone)]
pub enum FrameError {
    EmptyFrameError,
    InvalidErrorCode,
    LengthValueOutOfRange,
    NotEnoughBytes,
    RSSIValueOutOfRange,
    InvalidChecksum,
    InvalidCommandCode,
    InvalidCommandSequence,
    FailedToTransmitFrame,
    TransmissionTimedOut,
    ValueOutOfRange,
    InvalidUTF8,
    InvalidJson
}

pub struct FrameStack {
    local_frame_id: u32,
    tx_frame_queue: VecDeque<(u32, Frame)>,
    rx_frame_queue: VecDeque<(u32, Frame)>,

    remote_ackd_frame_id: u32,
}

impl FrameStack {
    pub fn new() -> FrameStack {
        FrameStack { 
            local_frame_id: 0,
            tx_frame_queue: VecDeque::new(),
            rx_frame_queue: VecDeque::new(),
            remote_ackd_frame_id: 0
        }
    }

    pub fn append_tx_frame(&mut self, frame: Frame) {
        self.tx_frame_queue.push_back((frame.frame_id, frame));
    }

    pub fn append_rx_frame(&mut self, frame: Frame) {
        self.rx_frame_queue.push_back((frame.frame_id, frame))
    }

    pub fn get_rx_frame_queue(&self) -> &VecDeque<(u32, Frame)> {
        &self.rx_frame_queue
    } 

    pub fn get_remote_ackd_frame_id (&self) -> u32 {
        self.remote_ackd_frame_id
    }

    pub fn set_remote_ackd_frame_id (&mut self, new_id: u32) {
        self.remote_ackd_frame_id = new_id;
    }

    pub fn curr_id(&self) -> u32 {
        self.local_frame_id
    }
}

impl Default for FrameStack {
    fn default() -> Self {
        Self::new()
    }
}

impl BSSID {
    pub fn new(bytes: [u8; 6]) -> Self {
        Self { bytes }
    }
    
    fn parse(bytes: &[u8]) -> Result<BSSID, FrameError> {
        if bytes.len() < 6 {
            return Err(FrameError::NotEnoughBytes);
        }
        let input = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]];

        Ok(BSSID { bytes: input })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<BSSID, FrameError> {
        BSSID::parse(bytes)
    }

    pub fn as_bytes(&self) -> [u8; 6] {
        self.bytes
    }
}

impl SSID {
    pub fn new(name: String) -> Self {
        Self { name }
    }
    
    pub fn parse(bytes: &[u8]) -> Result<SSID, FrameError> {
        let op = "INVALID_UTF-8";
        let name : String = from_utf8(bytes).unwrap_or(op).to_string();
        Ok(SSID { name })
    }
}

impl FromStr for SSID {
    type Err = FrameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SSID {name: s.to_string()})
    }

}

impl Checksum {
    pub fn from_bytes(bytes: &[u8]) -> Checksum {
        Checksum {
            checksum: CRC_16.checksum(bytes)
        }
    }

    pub fn from_int(checksum: u16) -> Checksum {
        Checksum { checksum }
    }

    pub fn check (&self, bytes: &[u8]) -> bool {
        CRC_16.checksum(bytes) == self.checksum
    }

    pub fn as_bytes(&self) -> [u8; 2] {
        self.checksum.to_be_bytes()
    }
}

impl RSSI {
    pub fn from_int(strength: i8) -> Result<RSSI, FrameError> {
        if !(-127..=0).contains(&strength) {
            Err(FrameError::RSSIValueOutOfRange)
        }
        else {
            Ok(RSSI {strength})
        }
    }

    pub fn as_bytes(&self) -> [u8; 1] {
        self.strength.to_be_bytes()
    }
}

impl Position {
    fn parse(bytes : &[u8]) -> Result<Position, FrameError> {
        if bytes.len() < 8 {
            return Err(FrameError::NotEnoughBytes);
        }
        let pitch: u32 = byte_slice_to_u32(&bytes[0..=3])?;
        let yaw  : u32 = byte_slice_to_u32(&bytes[4..=7])?;

        Ok(Position { pitch, yaw })
    }

    pub fn from_int(pitch: u32, yaw: u32) -> Position {
        Position {
            pitch,
            yaw
        }
    }

    pub fn from_degrees(pitch_deg: f32, yaw_deg: f32) -> Result<Position, FrameError> {
        if pitch_deg.is_nan() || yaw_deg.is_nan() {
            return Err(FrameError::ValueOutOfRange);
        }
        
        let pitch = constraint_to_degree(pitch_deg) / 360.0;
        let yaw   = constraint_to_degree(yaw_deg  ) / 360.0;

        let pitch = (pitch * u32::MAX as f32) as u32;
        let yaw   = (yaw   * u32::MAX as f32) as u32;

        Ok(Position {
            pitch, yaw
        })
    }

    pub fn as_bytes(&self) -> [u8; 8] {
        let pitch = self.pitch.to_be_bytes();
        let yaw = self.yaw.to_be_bytes();

        [pitch[0], pitch[1], pitch[2], pitch[3], yaw[0], yaw[1], yaw[2], yaw[3]]
    }
}

impl NetworkId {
    fn parse(bytes : &[u8]) -> Result<NetworkId, FrameError> {
        if bytes.len() < 4 {
            return Err(FrameError::NotEnoughBytes);
        }

        let id: u32 = byte_slice_to_u32(bytes)?;

        Ok(NetworkId { id })
    }

    pub fn from_int(id: u32) -> NetworkId {
        NetworkId {id}
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        self.id.to_be_bytes()
    }
}

impl Record {
    fn parse_multiple(count: u32 , bytes: &[u8]) -> Result<Vec<Record>, FrameError> {
        let mut read_vec = Vec::with_capacity(count as usize);
        for index in 0..count {
            let start = index as usize * 5;
            let end = (index as usize + 1) * 5;
            read_vec.push(Record::parse(&bytes[start..end])?);
        }

        Ok(read_vec)
    }

    fn parse(bytes: &[u8]) -> Result <Record, FrameError> {
        if bytes.len() < 5 {
            return Err(FrameError::NotEnoughBytes);
        }

        let rssi_bytes: [u8; 1] = [ bytes[4] ];
        Ok(Record {
            internal_id: NetworkId::parse(bytes)?,
            rssi: RSSI::from_int(i8::from_be_bytes(rssi_bytes))?,
        })
    }

    pub fn from_components(internal_id: NetworkId, rssi: RSSI) -> Record {
        Record { internal_id, rssi }
    }

    pub fn as_bytes(&self) -> [u8; 5] {
        let id = self.internal_id.as_bytes();
        let rssi = self.rssi.as_bytes();

        [id[0], id[1], id[2], id[3], rssi[0]]
    }

}

impl StepSize {
    fn parse(bytes: &[u8]) -> Result<StepSize, FrameError> {
        Ok(StepSize {
            pitch_step: byte_slice_to_u32(bytes)?,
            yaw_step  : byte_slice_to_u32(&bytes[4..])?
        })
    }

    pub fn from_pitch_yaw(pitch_step: u32, yaw_step: u32) -> Result<StepSize, FrameError> {
        
        Ok(StepSize { pitch_step, yaw_step })
    }

    pub fn from_degrees(pitch_deg: f32, yaw_deg: f32) -> Result<StepSize, FrameError> {
        if pitch_deg.is_nan() || yaw_deg.is_nan() {
            return Err(FrameError::ValueOutOfRange);
        }
        
        let pitch = constraint_to_degree(pitch_deg) / 360.0;
        let yaw   = constraint_to_degree(yaw_deg  ) / 360.0;

        let pitch_step = (pitch * u32::MAX as f32) as u32;
        let yaw_step   = (yaw   * u32::MAX as f32) as u32;

        Ok(StepSize {
            pitch_step, yaw_step
        })
    }

    pub fn pitch(&self) -> u32 {
        self.pitch_step
    }

    pub fn yaw(&self) -> u32 {
        self.yaw_step
    }

    pub fn as_bytes(&self) -> [u8; 8] {
        let pitch = u32::to_be_bytes(self.pitch_step);
        let yaw   = u32::to_be_bytes(self.yaw_step  );

        [pitch[0], pitch[1], pitch[2], pitch[3], yaw[0], yaw[1], yaw[2], yaw[3]]
    }
}


impl Cmd {

    pub fn parse_body(cmd_nibble: u8, length: u16, data: &[u8]) -> Result <Cmd, FrameError> {

        match cmd_nibble {
            0x0 => if length != 0 { Err(FrameError::LengthValueOutOfRange)} else { Ok(Cmd::StartOfTransmission) },
            0x1 => if length != 0 { Err(FrameError::LengthValueOutOfRange)} else { Ok(Cmd::Reset              ) },
            0x2 => if length != 0 { Err(FrameError::LengthValueOutOfRange)} else { Ok(Cmd::Ready              ) },
            0x3 => if length != 0 { Err(FrameError::LengthValueOutOfRange)} else { Ok(Cmd::RequestPosition    ) },
            0xF => if length != 0 { Err(FrameError::LengthValueOutOfRange)} else { Ok(Cmd::EndOfTransmission  ) },
            
            0x4|0x6 => {
                if length != 0x004 {
                    return Err(FrameError::LengthValueOutOfRange);
                }

                let id = byte_slice_to_u32(data)?;

                match cmd_nibble {
                    0x4 => Ok(Cmd::Ack               { frame_id: id }),
                    0x6 => Ok(Cmd::RequestAck        { frame_id: id }),
                    _ => panic!("Unrechable!")
                }
            },

            0x5 => {
                if length != 0x008 {
                    return Err(FrameError::LengthValueOutOfRange);
                }

                let start = byte_slice_to_u32(data)?;
                let end   = byte_slice_to_u32(&data[4..])?;

                Ok(Cmd::RequestRetransmit { frame_id_start: start, frame_id_end: end })
            },
            
            0x7 => {
                if !(0x4..=0x24).contains(&length) {
                    return Err(FrameError::LengthValueOutOfRange);
                }
                let id = NetworkId::parse(&data[0..4])?;
                let ssid =
                    if length == 0x4 {
                        SSID::from_str("")?
                    }
                    else {
                        SSID::parse(&data[4..])?
                    };

                Ok(Cmd::AddSSID { id, ssid })
            },


            0x8 => {
                if length != 0xA {
                    return Err(FrameError::LengthValueOutOfRange)
                }

                Ok(Cmd::AddBSSID {
                        id   : NetworkId::parse(&data[0..4])?,
                        bssid: BSSID::parse(&data[4..])?
                    }
                )
            }

            0x9 => {
                if length < 0x11 {
                    return Err(FrameError::LengthValueOutOfRange);
                }
                let count = byte_slice_to_u32(&data[8..=11])?;
                Ok(Cmd::RecordRSSI {
                        position: Position::parse(&data[0..=7])?,
                        record_count: count,
                        records: Record::parse_multiple(count, &data[12..])?
                    }
                )
            },

            0xA => {
                if length != 8 {
                    return Err(FrameError::LengthValueOutOfRange);
                }

                Ok(Cmd::SetPosition {
                        position: Position::parse(&data[0..=7])?
                    }
                )
            },
        
            0xB =>  {
                if length != 0x11 {
                    return Err (FrameError::LengthValueOutOfRange);
                }

                Ok (Cmd::SetParams {
                        position: Position::parse(&data[0..=7])?,
                        step_size: StepSize::parse(&data[8..=15])?,
                        measurements_per_step: data[16]
                    }
                )
            },

            0xC => {
                if length < 0x00A {
                    return Err(FrameError::LengthValueOutOfRange);
                }
                
                if let Ok(body_str) = std::str::from_utf8(&data[8..]) {
                    if let Ok(body) =  json::from_str(body_str) {

                        return Ok(Cmd::TransmitPicture { 
                            position: Position::parse(&data[0..=7])?,
                            body
                        });
                    }
                    return Err(FrameError::InvalidJson);
                }

                Err(FrameError::ValueOutOfRange)
                
            }

            0xD => {
                
                if length < 0x002 {
                    dbg!(&data);
                    return Err(FrameError::LengthValueOutOfRange);
                }

                if let Ok(logs) = std::str::from_utf8(&data) {
                    match json::from_str(logs) {
                        Ok(logs) => return Ok(Cmd::TransmitLogs { logs }),
                        Err(_) => {
                            return Err(FrameError::InvalidJson);
                        }
                    }
                    
    
                }

                Err(FrameError::InvalidUTF8)
            }

            _ => Err(FrameError::InvalidCommandCode)
        }

    }

    pub fn parse(cmd_nibble: u8, length: u16, frame_length: u16, bytes: &[u8]) -> Result<Cmd, FrameError> {    
        if (0x7..=0xB).contains(&cmd_nibble) && usize::from(length) > bytes.len() - FRAME_HEADER_SIZE {
            return Err(FrameError::NotEnoughBytes);
        }

        let data = &bytes[FRAME_HEADER_SIZE..(frame_length as usize - CHECKSUM_SIZE)];

        let cmd = Cmd::parse_body(cmd_nibble, length, data)?;
        
        Ok( cmd  )
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, FrameError> {
        match self {
            Cmd::StartOfTransmission |
            Cmd::Reset               |
            Cmd::Ready               |
            Cmd::RequestPosition     |
            Cmd::EndOfTransmission   => Ok(vec![]),

            Cmd::Ack                 {frame_id} |
            Cmd::RequestAck          {frame_id}  => {
                Ok(u32::to_be_bytes(*frame_id).to_vec())
            }

            Cmd::RequestRetransmit   {frame_id_start, frame_id_end} => {
                let start = u32::to_be_bytes(*frame_id_start);
                let end   = u32::to_be_bytes(*frame_id_end  );
                Ok(vec![start[0], start[1], start[2], start[3], end[0], end[1], end[2], end[3]])
            }

            Cmd::AddSSID     { id, ssid } => {
                let id = id.as_bytes();
                let ssid = ssid.name.as_bytes();
                let mut result = Vec::with_capacity(id.len() + ssid.len());
                
                result.extend(&id);
                result.extend(ssid);
                Ok(result)
            },
            Cmd::AddBSSID    { id, bssid } => {
                let id = id.as_bytes();
                let bssid = bssid.as_bytes();
                let mut result = Vec::with_capacity(id.len() + bssid.len());
                result.extend_from_slice(&id);
                result.extend_from_slice(&bssid);

                Ok(result)
            },
            Cmd::RecordRSSI  { position, record_count, records } => {
                let position = position.as_bytes();
                let record_count = record_count.to_be_bytes();

                let mut result = Vec::with_capacity(position.len() + record_count.len() + records.len() * 5);
                result.extend_from_slice(&position);
                result.extend_from_slice(&record_count);
                for record in records {
                    result.extend_from_slice(&record.as_bytes());
                }

                Ok(result)
            },
            Cmd::SetPosition { position } => {
                let position = position.as_bytes();
                Ok(position.to_vec())
            },
            Cmd::SetParams   { position, step_size, measurements_per_step } => {
                let position  = position.as_bytes();
                let step_size = step_size.as_bytes();
                
                let mut result = Vec::with_capacity(position.len() + step_size.len() + 1);
                result.extend_from_slice(&position);
                result.extend_from_slice(&step_size);
                result.push(*measurements_per_step);

                Ok(result)

            },
            Cmd::TransmitPicture { position, body } => {
                let position = position.as_bytes();
                let body = body.to_string();

                let mut result = Vec::with_capacity(position.len() + body.len());
                result.extend_from_slice(&position);
                result.extend_from_slice(body.as_bytes());

                Ok(result)
            },
            Cmd::TransmitLogs { logs } => {
                let result = logs.to_string().as_bytes().to_vec();

                Ok(result)
            }
            
        }
    }

    pub fn as_int(&self) -> Result<u8, FrameError> {
        #[allow(unreachable_patterns)]
        match self {
            Cmd::StartOfTransmission       => Ok(0x00),
            Cmd::Reset                     => Ok(0x01),
            Cmd::Ready                     => Ok(0x02),
            Cmd::RequestPosition           => Ok(0x03),
            Cmd::Ack               { .. }  => Ok(0x04),
            Cmd::RequestRetransmit { .. }  => Ok(0x05),
            Cmd::RequestAck        { .. }  => Ok(0x06),
            Cmd::AddSSID           { .. }  => Ok(0x07),
            Cmd::AddBSSID          { .. }  => Ok(0x08),
            Cmd::RecordRSSI        { .. }  => Ok(0x09),
            Cmd::SetPosition       { .. }  => Ok(0x0A),
            Cmd::SetParams         { .. }  => Ok(0x0B),
            Cmd::TransmitPicture   { .. }  => Ok(0x0C),
            Cmd::TransmitLogs      { .. }  => Ok(0x0D),
            Cmd::EndOfTransmission         => Ok(0x0F),
            _ => Err(FrameError::InvalidCommandCode)
        }
    }

}

impl Frame {
    pub fn parse_header(bytes: &[u8]) -> Result <(u8, u16, u16, u32), FrameError> {
        if bytes.is_empty() {
            return Err(FrameError::EmptyFrameError);
        }

        if bytes.len() < FRAME_HEADER_SIZE {
            return Err(FrameError::NotEnoughBytes)
        }

        let cmd_nibble: u8  = (bytes[0] & 0xF0) >> 4;

        let (length, frame_length) = {
                
                // TODO: Convert to from_be_bytes
                let l = u16::from_be_bytes([bytes[0], bytes[1]]) & 0x0FFF;

                (l, l + (FRAME_HEADER_SIZE + CHECKSUM_SIZE) as u16 /* 2 cmd+length, 4 frame_id, 2 checksum */)
                
            };

            let frame_id :u32 = byte_slice_to_u32(&bytes[2..])?;

        Ok((cmd_nibble, length, frame_length, frame_id))
    }

    pub fn parse(bytes: &[u8]) -> Result<(Frame, u16), FrameError> {
        let (cmd_nibble, length, frame_length, frame_id) = Self::parse_header(bytes)?;

        let cmd = Cmd::parse(cmd_nibble, length, frame_length, bytes)?;

        let consumed = FRAME_HEADER_SIZE as u16 + length;

        let checksum = Checksum::from_int( byte_slice_to_u16(&bytes[consumed as usize..])?);

        if checksum.check(&bytes[0..consumed as usize]) {
            Ok((Frame {cmd, frame_id, checksum}, consumed + CHECKSUM_SIZE as u16))
        } else {
            Err(FrameError::InvalidChecksum)
        }
    }

    pub fn from_cmd(cmd: Cmd, frame_id: u32) -> Result<Frame, FrameError> {
        let mut body    : Vec<u8> = cmd.as_bytes()?;
        let mut header  : Vec<u8> = Frame::header_as_bytes(cmd.as_int()?, body.len() as u16, frame_id).to_vec();

        header.append(&mut body);

        let checksum = Checksum::from_bytes(&header);
        Ok(Frame { cmd, frame_id, checksum })
    }

    pub fn from_components(cmd: Cmd, frame_id: u32, checksum: Checksum) -> Result<Frame, FrameError> {
        Ok(Frame{ cmd, frame_id, checksum })
    }

    fn header_as_bytes(cmd_nibble: u8, length: u16, frame_id: u32) ->[u8; FRAME_HEADER_SIZE] {
        let frame = u32::to_be_bytes(frame_id);
        [cmd_nibble << 4 | ((length & 0x0F00) >> 8) as u8, (length & 0x00FF) as u8, frame[0], frame[1], frame[2], frame[3]]
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, FrameError> {
        let mut body    : Vec<u8> = self.cmd.as_bytes()?;
        let mut checksum: Vec<u8> = self.checksum.as_bytes().to_vec(); 
        let mut header  : Vec<u8> = Frame::header_as_bytes(self.cmd.as_int()?, body.len() as u16, self.frame_id).to_vec();

        let mut result : Vec<u8> = Vec::with_capacity(FRAME_HEADER_SIZE + body.len() + CHECKSUM_SIZE);

        result.append(&mut header  );
        result.append(&mut body    );
        result.append(&mut checksum);

        Ok(result)
    }

    pub fn get_cmd(&self) -> &Cmd {
        &self.cmd
    }
}
