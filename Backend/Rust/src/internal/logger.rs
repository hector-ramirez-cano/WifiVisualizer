use std::sync::{Arc, Mutex};
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

    pub fn log(&mut self, severity: Severity, msg: String) {
        self.logs.push(Log {severity, msg});
    }

    pub fn get_logs(&self) -> &Vec<Log> {
        &self.logs
    }
}

pub fn log(logger : &mut Arc<Mutex<Logger>>, severity : Severity, msg: &str) {
    if let Ok(mut handle) = logger.lock() {
        handle.log(severity, msg.to_string());
    }
}