const ctx = new AudioContext({
    sampleRate: 44100,
    latencyHint: "interactive",
})

if (ctx.audioWorklet === undefined) {
    alert("AudioWorklet isn't supported... It cannot work.")
} else {  
    ctx.audioWorklet.addModule('js/worklet.js?t=' + new Date().getTime())
	.then(() => {
	    const n = new AudioWorkletNode(ctx, 'ruffbox-processor', { numberOfInputs: 1,
								       numberOfOutputs: 1,
								       outputChannelCount: [2], } );
	    n.connect(ctx.destination)
	    	    	   
	    ///////////////////////////	    
	    // SET KEYBOARD TRIGGERS //
	    ///////////////////////////
	    
	    window.onkeyup = function(e) {
		var key = e.keyCode ? e.keyCode : e.which;
				
	    }
	    
	    ///////////////////////////////
	    // INITIALIZE SAMPLER MODULE //
	    ///////////////////////////////
	    
	    fetch('wasm/ruffbox_sampler.wasm?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => n.port.postMessage({ type: 'loadWasm', data: r }))

	    //////////////////
	    // LOAD SAMPLES //
	    //////////////////
	    
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
	    
	    // this only works in latest chrome with experimental features flag enabled ...
	    let scheduler = new Worker('js/scheduler.js', { type : "module"});

	    // set event scheduler message callback
	    scheduler.onmessage = function(event) {
		// dispatch to sampler module ...
		n.port.postMessage({ type: 'trigger', sample_id: event.data})
	    };

	    ///////////////////////////
	    // INIT CONTROL ELEMENTS //
	    ///////////////////////////
	    
	    const bdTrig = document.getElementById('bassdrum-trigger')
	    bdTrig.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    n.port.postMessage({ type: 'trigger', sample_id: 'bd'})
		}
	    })

	    const snTrig = document.getElementById('snare-trigger')
	    snTrig.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    n.port.postMessage({ type: 'trigger', sample_id: 'sn'})
		}
	    })

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
		    scheduler.postMessage({ cmd: 'start' });
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
	})   
}
