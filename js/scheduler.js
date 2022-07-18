import init, { Scheduler } from './pkg/ruffbox_pattern.js';

async function get_scheduler() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.    
    
    await init();
    
    return Scheduler.new();	  
}

// In a previous version, more of this was done in the Rust part using
// stdweb's `js!` snippet macro, but that stopped working and didn't
// seem to be very elegant.
// using web_sys and js_sys to patch everything together in rust
// also seems more hassle than it's worth, so until wasm_thread allows to call
// postMessage, or std::thread becomes available so that all the scheduling
// can be done on the rust side, with completely invisible web workers,
// this solution seems to be the best compromise to me. 
function time_step(start, init_time) {
    const current_time = performance.now();
    if (start === true) {
	self.scheduler.start(init_time, current_time);
    }

    // generate the event list ...
    let next_events = self.scheduler.generate_events();

    // dispatch events to audio worklet ..
    next_events.forEach((event) => {
	postMessage( {
	    source_type: event.source_type,
	    timestamp: event.timestamp,
	    sample_id: event.sample_id,
	    params: event.params } );	
    });

    // time compensation is necessary because the setTimeout function
    // isn't all that precise ... the real time thread in the worklet is
    // running a bit ahead so all the events have time to arrive 
    const next_time_step = self.scheduler.compensate_time(current_time);
    self.sched_timeout = setTimeout(time_step, next_time_step, false);
}

// fetch the scheduler instance
get_scheduler().then(scheduler => {
    self.scheduler = scheduler;

    // now that we have a scheduler, set scheduler controls
    self.onmessage = function(e) {
	console.log("scheduler command: " + e.data.cmd);
	switch (e.data.cmd) {
	case 'start':	    
	    time_step(true, e.data.timestamp);
	    break;
	case 'stop':
	    clearTimeout(self.sched_timeout);	    
	    break;
	case 'evaluate_loop':
	    self.scheduler.evaluate(e.data.loop_data);
	    break;
	case 'set_tempo':
	    self.scheduler.set_tempo(e.data.tempo);
	    break;
	}
    }
});




