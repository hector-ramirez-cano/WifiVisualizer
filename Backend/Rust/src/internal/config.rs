use std::{fs::File, io::Read, net::{IpAddr, Ipv4Addr}, str::FromStr};

use rocket::serde::json;


pub struct Config {
    esp32_cam_ip: IpAddr
}

impl Config {
    pub fn esp32_cam_ip(&self) -> IpAddr {
        self.esp32_cam_ip
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { esp32_cam_ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 1)) }
    }
}

pub fn load_config() -> Option<Config> {
    let config = if let Ok(mut file) = File::open("res/config.json") {
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        contents
    } else {
        String::from("{}")
    };

    let config = json::from_str(&config).unwrap_or(json::json!({}));
    let config = config.as_object()?;
    
    let esp32_cam = config.get("esp32_cam")?.as_object()?;
    let esp32_cam_ip = esp32_cam.get("ip")?.as_str()?;

    Some(Config {
        esp32_cam_ip: IpAddr::V4(Ipv4Addr::from_str(esp32_cam_ip).ok()?),
    })
}