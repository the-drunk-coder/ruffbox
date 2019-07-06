#[macro_use]
extern crate stdweb;
extern crate web_sys;

use wasm_bindgen::prelude::*;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct Scheduler {
    running: bool,
    tempo: i32,
    event_idx: usize,
    event_sequence: Vec<String>,
}

#[wasm_bindgen]
impl Scheduler {
    pub fn new() -> Self {
        Scheduler{
            running: false,
            tempo: 512,
            event_idx: 0,
            event_sequence: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, input: Option<String>) {
        self.event_sequence.clear();
        
        match input {
            Some(line) => {
                let iter = line.split_ascii_whitespace();

                for event in iter {
                    self.event_sequence.push(event.to_string());
                }
            }
            None => log!("no input!")
        }        
    }

    
    pub fn scheduler_routine(&mut self) {
        if !self.running {
            return
        }

        let next_event = &self.event_sequence[self.event_idx];
        
        if self.event_idx + 1 == self.event_sequence.len() {
            self.event_idx = 0;
        } else {
            self.event_idx += 1;
        }
        
        js! {
            // post events that will be dispatched to sampler
            postMessage( @{next_event} );
            // time-recursive call to scheduler function
            self.sleep( @{self.tempo} ).then( () => self.scheduler.scheduler_routine() );
        };                
    }

    pub fn start(&mut self) {
        self.running = true;
        self.scheduler_routine();
    }
    
    pub fn stop(&mut self) {
        self.running = false;
    }
}
