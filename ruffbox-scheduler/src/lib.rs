#[macro_use]
extern crate stdweb;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Scheduler {}

#[wasm_bindgen]
impl Scheduler {
    pub fn new() -> Self {
        Scheduler{}
    }

    pub fn scheduler_routine(&self) {
        
        js! {
            // post events that will be dispatched to sampler
            postMessage("hi");
            // time-recursive call to scheduler function
            self.sleep(2000).then( () => self.scheduler.scheduler_routine() );
        };                
    }
}
