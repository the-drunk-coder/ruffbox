importScripts('./pkg/ruffbox_pattern.js')

async function load_scheduler() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.    
    await wasm_bindgen('./pkg/ruffbox_pattern_bg.wasm');

    // now that we have a scheduler, set scheduler controls
    self.onmessage = async e => {
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
	case 'set_tempo':
	    self.scheduler.set_tempo(e.data.tempo);
	    break;
	}
    };        
};

load_scheduler();




