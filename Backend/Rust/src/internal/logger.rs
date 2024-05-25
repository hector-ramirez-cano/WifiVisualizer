use rocket::serde::{Serialize, Deserialize};


#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum Severity {
    ALL,
    VERBOSE,
    DEBUG,
    INFO,
    WARNING,
    ERROR,
}

impl Serialize for Severity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: rocket::serde::Serializer {
        serializer.serialize_u8(self.value())
    }
}

impl Severity {
    pub fn value(&self) -> u8 {
        match self {
            Severity::ALL     => 0,
            Severity::VERBOSE => 1,
            Severity::DEBUG   => 2,
            Severity::INFO    => 3,
            Severity::WARNING => 4,
            Severity::ERROR   => 5,
        }
    }
}

impl TryFrom<u64> for Severity {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            value if value == Severity::VERBOSE as u64 => Ok(Severity::VERBOSE),
            value if value == Severity::DEBUG   as u64 => Ok(Severity::DEBUG  ),
            value if value == Severity::INFO    as u64 => Ok(Severity::INFO   ),
            value if value == Severity::WARNING as u64 => Ok(Severity::WARNING),
            value if value == Severity::ERROR   as u64 => Ok(Severity::ERROR  ),
            _ => Err(())
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Log {
    severity: Severity,
    msg: String
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Logger {
    logs: Vec<Log>
}

impl Logger {
    pub fn new() -> Logger {
        Logger {
            logs: vec![]
        }
    }

    pub fn log(&mut self, severity: Severity, msg: &str) {
        self.logs.push(Log {severity, msg : msg.to_string()});
    }

    pub fn get_logs(&self) -> &Vec<Log> {
        &self.logs
    }
}