pub struct Probe;

impl Probe {
    pub fn new() -> Self {
        Self
    }

    pub fn level(&mut self, _tick: u64) -> Option<u8> {
        None
    }
}
