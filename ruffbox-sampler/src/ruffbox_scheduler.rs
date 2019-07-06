pub mod ruffbox;

use ruffbox::*;

use std::thread;

pub struct RuffboxScheduler {
    pub ruffbox: Ruffbox,
}

impl RuffboxScheduler {
    pub fn new() -> RuffboxScheduler {
        RuffboxScheduler {
            ruffbox: Ruffbox::new(),
        }
    }

    pub fn startSchedulerThread(&self) {
        thread::spawn(move || {
            // some work here
        });
    }
    
}
