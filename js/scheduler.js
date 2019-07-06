import init, { Scheduler } from './pkg/ruffbox_scheduler.js';

async function get_scheduler() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.    
    
    await init();

    return Scheduler.new();	  
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

self.sleep = sleep;

get_scheduler().then(scheduler => {
    self.scheduler = scheduler;
    self.scheduler.scheduler_routine();
});


