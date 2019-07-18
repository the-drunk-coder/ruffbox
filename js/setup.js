const sourceType = Object.freeze({
    "Sampler" : 0,
    "SineOsc" : 1,
    "SineSynth" : 2,
});

const sourceParameter = Object.freeze({
    "Attack" : 0,
    "Decay" : 1,
    "Duration" : 2,
    "PitchFrequency" : 3,
    "PitchNote" : 4,
    "HighpassCutoffFrequency" : 5,
    "HighpassQFactor" : 6,
    "Level" : 7,
    "LowpassCutoffFrequency" : 8,
    "LowpassQFactor" : 9,
    "LowpassFilterDistortion" : 10,
    "PeakFrequency" : 11,
    "PeakGain" : 12,
    "PeakQFactor" : 13,
    "Pulsewidth" : 14,
    "PlaybackRate" : 15,
    "PlaybackStart" : 16,
    "PlaybackLoop" : 17,
    "Release" : 18,
    "ReverbMix" : 19,
    "SampleBufferNumber" : 20,
    "Samplerate" : 21,
    "StereoPosition" : 22,
    "Sustain" : 23,
});

const sourceParameterShortName = Object.freeze({
    "atk" : 0,
    "dec" : 1,
    "dur" : 2,
    "freq" : 3,
    "note" : 4,
    "hp-freq" : 5,
    "hp-q" : 6,
    "lvl" : 7,
    "lp-freq" : 8,
    "lp-q" : 9,
    "lp-dist" : 10,
    "pf-freq" : 11,
    "pf-gain" : 12,
    "pf-q" : 13,
    "pw" : 14,
    "rate" : 15,
    "start" : 16,
    "loop" : 17,
    "rel" : 18,
    "rev" : 19,
    "buf" : 20,
    "sr" : 21,
    "pos" : 22,
    "sus" : 23,
});


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
	    fetch('wasm/ruffbox_synth.wasm?t=' + new Date().getTime())
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

	    // fetch sample files, decode and post them to worklet ...
	    fetch('audio/hh.flac?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => ctx.decodeAudioData(r)
		      .then(r => n.port.postMessage({ type: 'loadSample', samples: r.getChannelData(0), length: r.length, sample_id: 'hh' })))

	    fetch('audio/casio.flac?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => ctx.decodeAudioData(r)
		      .then(r => n.port.postMessage({ type: 'loadSample', samples: r.getChannelData(0), length: r.length, sample_id: 'casio' })))

	    //////////////////////////
	    // LOAD EVENT SCHEDULER //
	    //////////////////////////
	    
	    // this only works in latest chrome/chromium with experimental features flag enabled ...
	    let scheduler = new Worker('js/scheduler.js', { type : "module"});

	    // set event scheduler message callback
	    scheduler.onmessage = function(event) {
		// dispatch to sampler module ...
		n.port.postMessage({ type: 'trigger', event: event.data })
	    };

	    ///////////////////
	    // LOAD MAPPINGS //
	    ///////////////////

	    n.port.postMessage({ type: 'sourceType', content: sourceType});
	    n.port.postMessage({ type: 'sourceParameter', content: sourceParameterShortName});
	    	    
	    ///////////////////////////
	    // INIT CONTROL ELEMENTS //
	    ///////////////////////////

	    var running = false;
	    var tempo = document.getElementById('tempo').value
	    
	    // manual triggers
	    const bdTrig = document.getElementById('bassdrum-trigger')
	    bdTrig.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    n.port.postMessage({ type: 'trigger', event : { source_type : "Sampler", sample_id: 'bd', timestamp: 0.0 }})
		}
	    })

	    const snTrig = document.getElementById('snare-trigger')
	    snTrig.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    n.port.postMessage({ type: 'trigger', event : { source_type : "Sampler", sample_id: 'sn', timestamp: 0.0 }})
		}
	    })
	  
	    const startSched = document.getElementById('start-scheduler')	   
	    startSched.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    if(ctx.state === "suspended"){
			ctx.resume();
		    }
		    if(!(document.getElementById('tempo').value === tempo)){
			tempo = document.getElementById('tempo').value;
			scheduler.postMessage({ cmd: 'set_tempo' , tempo: tempo });
		    }
		    scheduler.postMessage({ cmd: 'evaluate_loop' , loop_data: document.getElementById('code_input').value });
		    scheduler.postMessage({ cmd: 'start', timestamp: ctx.currentTime });
		    running = true;
		} else {
		    scheduler.postMessage({ cmd: 'stop' });
		    running = false;
		}
	    })

	    const evalLoop = document.getElementById('evaluate-loop')
	    evalLoop.addEventListener('change', e => {				
		if (e.target.value === 1) {
		    console.log(document.getElementById('tempo').value)
		    if(!(document.getElementById('tempo').value === tempo)){
			tempo = document.getElementById('tempo').value;
			scheduler.postMessage({ cmd: 'set_tempo' , tempo: tempo });
		    }
		    scheduler.postMessage({ cmd: 'evaluate_loop' , loop_data: document.getElementById('code_input').value });
		} 
	    })

	    ///////////////////////////	    
	    // SET KEYBOARD TRIGGERS //
	    ///////////////////////////
	    
	    window.onkeydown = function(e) {
		var key = e.keyCode ? e.keyCode : e.which;
		if(e.ctrlKey && key == 13) {
		    if(!(document.getElementById('tempo').value === tempo)){
			tempo = document.getElementById('tempo').value;
			scheduler.postMessage({ cmd: 'set_tempo' , tempo: tempo });
		    }
		    scheduler.postMessage({ cmd: 'evaluate_loop' , loop_data: document.getElementById('code_input').value });
		} else if(e.ctrlKey && key == 190) {
		    if(!running){
			if(ctx.state === "suspended"){
			    ctx.resume();
			}			
			scheduler.postMessage({ cmd: 'evaluate_loop' , loop_data: document.getElementById('code_input').value });
			scheduler.postMessage({ cmd: 'start', timestamp: ctx.currentTime });
			document.getElementById('start-scheduler').value = 1;
			running = true;
		    } else {
			scheduler.postMessage({ cmd: 'stop' });			
			running = false;
			document.getElementById('start-scheduler').value = 0;
		    }
		}
	    }
	})   
}
