const ctx = new AudioContext({
    sampleRate: 44100,
    latencyHint: "interactive",
})

if (ctx.audioWorklet === undefined) {
    alert("AudioWorklet isn't supported... It cannot work.")
} else {  
    ctx.audioWorklet.addModule('js/worklet.js?t=' + new Date().getTime())
	.then(() => {

	    ///////////////////////////////
	    // INITIALIZE SAMPLER MODULE //
	    ///////////////////////////////

	    // first, get the audio worklet node of the current context ...
	    const n = new AudioWorkletNode(ctx, 'ruffbox-processor', { numberOfInputs: 1,
								       numberOfOutputs: 1,
								       outputChannelCount: [2], } );
	    // connect to output to activate ...
	    n.connect(ctx.destination);
	    	    	   
	    // now, get the WASM module file and post the raw binary to
	    // the worklet, where it'll be instantiated.
	    // unfortunately, as of now, the audio worklet doesn't allow fetching
	    // the WASM module directly
	    fetch('wasm/ruffbox_sampler.wasm?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => n.port.postMessage({ type: 'loadWasm', data: r }))

	    //////////////////
	    // LOAD SAMPLES //
	    //////////////////

	    // fetch sample files, decode and post them to worklet ...
	    fetch('audio/bd.flac?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => ctx.decodeAudioData(r)
		      .then(r => n.port.postMessage({ type: 'loadSample', samples: r.getChannelData(0), length: r.length, sample_id: 'bd' })))

	    fetch('audio/sn.flac?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => ctx.decodeAudioData(r)
		      .then(r => n.port.postMessage({ type: 'loadSample', samples: r.getChannelData(0), length: r.length, sample_id: 'sn' })))

	    //////////////////////////
	    // LOAD EVENT SCHEDULER //
	    //////////////////////////
	    
	    // this only works in latest chrome/chromium with experimental features flag enabled ...
	    let scheduler = new Worker('js/scheduler.js', { type : "module"});

	    // set event scheduler message callback
	    scheduler.onmessage = function(event) {
		// dispatch to sampler module ...
		n.port.postMessage({ type: 'trigger', sample_id: event.data.sample, timestamp: event.data.timestamp})
	    };
	    	    
	    ///////////////////////////
	    // INIT CONTROL ELEMENTS //
	    ///////////////////////////

	    // manual triggers
	    const bdTrig = document.getElementById('bassdrum-trigger')
	    bdTrig.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    n.port.postMessage({ type: 'trigger', sample_id: 'bd', timestamp: 0.0})
		}
	    })

	    const snTrig = document.getElementById('snare-trigger')
	    snTrig.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    n.port.postMessage({ type: 'trigger', sample_id: 'sn', timestamp: 0.0})
		}
	    })

	    // scheduler control
	    const startCtx = document.getElementById('start-audio')
	    startCtx.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    ctx.resume();
		} else {
		    ctx.suspend();
		}
	    })

	    const startSched = document.getElementById('start-scheduler')
	    startSched.addEventListener('change', e => {				
		if (e.target.value === 1) {		    
		    scheduler.postMessage({ cmd: 'start', timestamp: ctx.currentTime });		    
		} else {
		    scheduler.postMessage({ cmd: 'stop' });
		}
	    })

	    const evalLoop = document.getElementById('evaluate-loop')
	    evalLoop.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    scheduler.postMessage({ cmd: 'evaluate_loop' , loop_data: document.getElementById('code_input').value });
		} 
	    })

	    ///////////////////////////	    
	    // SET KEYBOARD TRIGGERS //
	    ///////////////////////////
	    
	    window.onkeydown = function(e) {
		var key = e.keyCode ? e.keyCode : e.which;				
	    }
	})   
}
