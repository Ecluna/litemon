pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;

use sysinfo::{System, SystemExt};

pub struct Monitor {
    sys: System,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }
} 