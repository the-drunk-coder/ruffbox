import init, { Scheduler } from './pkg/ruffbox_scheduler.js';

async function get_scheduler() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.    
    
    await init();
    
    return Scheduler.new();	  
}

// not precise, just for testing purposes ...
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

self.sleep = sleep;

get_scheduler().then(scheduler => {
    self.scheduler = scheduler;

    // now that we have a scheduler, set scheduler controls ...
    self.onmessage = function(e) {
	console.log("scheduler command: " + e.data.cmd);
	switch (e.data.cmd) {
	case 'start':
	    self.scheduler.start(e.data.timestamp, performance.now());
	    break;
	case 'stop':
	    self.scheduler.stop();
	    break;
	case 'evaluate_loop':
	    self.scheduler.evaluate(e.data.loop_data);
	    break;
	}
    }
});




