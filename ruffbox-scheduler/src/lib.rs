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
    audio_start_time: f64,
    browser_start_time: f64,
    audio_logical_time: f64,
    browser_logical_time: f64,
    next_schedule_time: f64,
    lookahead: f64, // in seconds
    running: bool,
    tempo: f64, // currently just the duration of a 16th note ... 
    event_idx: usize,
    event_sequence: Vec<String>,
}

#[wasm_bindgen]
impl Scheduler {
    pub fn new() -> Self {
        Scheduler{
            audio_start_time: 0.0,
            browser_start_time: 0.0,
            audio_logical_time: 0.0,
            browser_logical_time: 0.0,
            next_schedule_time: 0.0,
            lookahead: 0.100,
            running: false,
            tempo: 128.0,
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

    fn generate_and_send_events(&mut self) {
        if self.event_sequence.is_empty() {
            return
        }
        
        let next_event = &self.event_sequence[self.event_idx];
        
        if self.event_idx + 1 == self.event_sequence.len() {
            self.event_idx = 0;
        } else {
            self.event_idx += 1;
        }

        let trigger_time = self.audio_logical_time + self.lookahead;
        
        if next_event != "~" {
            // post events that will be dispatched to sampler
            js! {                
                postMessage( { sample: @{ next_event }, timestamp: @{ trigger_time } } );
            }
        }
    }
    
    pub fn scheduler_routine(&mut self, browser_timestamp: f64) {
        if !self.running {
            return
        }
                
        self.generate_and_send_events();

        // calculate drift
        self.next_schedule_time = self.tempo - (browser_timestamp - self.browser_logical_time);

        // advance timestamps
        // audio in seconds
        self.audio_logical_time += self.tempo / 1000.0;

        // browser in milliseconds
        self.browser_logical_time += self.tempo;
                
        // time-recursive call to scheduler function
        js! {            
            self.sleep( @{ self.next_schedule_time } ).then( () => self.scheduler.scheduler_routine( performance.now()));
        };                
    }

    pub fn start(&mut self, audio_timestamp: f64, browser_timestamp: f64) {
        self.audio_start_time = audio_timestamp;
        self.browser_start_time = browser_timestamp;
        self.audio_logical_time = self.audio_start_time;
        self.browser_logical_time = self.browser_start_time;
        self.running = true;
        self.scheduler_routine(browser_timestamp);
    }
    
    pub fn stop(&mut self) {
        self.running = false;
    }
}
