use serde::{Serialize, Deserialize};

use super::{Device, Mode};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RTU {
    pub name: String,
    pub id: String,
    pub devices: Vec<Device>
}

impl RTU {
    pub async fn update(rtu: &mut RTU, mode: &Mode) {
        for mut device in &mut rtu.devices {
            Device::update(&mut device, &mode).await;
        }
    }
}