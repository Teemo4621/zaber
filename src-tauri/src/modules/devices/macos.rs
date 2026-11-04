use super::{Device, Kind};

pub struct Probe;

impl Probe {
    pub fn new() -> Self {
        Self
    }

    pub fn devices(&mut self, _tick: u64) -> Vec<Device> {
        Vec::new()
    }

    pub fn batteries(&mut self) -> Vec<(String, Kind, String, u8)> {
        Vec::new()
    }
}
