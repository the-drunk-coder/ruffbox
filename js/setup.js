const {startup} = wasm_bindgen;

async function run_wasm() {
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`
    // `wasm_bindgen` was imported in `index.html`
    await wasm_bindgen('/js/pkg/ruffbox_pattern_bg.wasm');
    
    console.log('ruffbox_pattern_bg loaded');
    
    // Run main WASM entry point
    // This will create a worker from within our Rust code compiled to WASM
    startup();
}

const sourceType = Object.freeze({
    "Sampler" : 0,
    "SineOsc" : 1,
    "SineSynth" : 2,
    "LFSawSynth" : 3,
    "LFSquareSynth" : 4,
});

const sourceParameter = Object.freeze({
    "Attack" : 0,
    "Decay" : 1,
    "DelayDampeningFrequency" : 2,
    "DelayFeedback" : 3,
    "DelayMix" : 4,
    "DelayTime" : 5,
    "Duration" : 6,
    "PitchFrequency" : 7,
    "PitchNote" : 8,
    "HighpassCutoffFrequency" : 9,
    "HighpassQFactor" : 10,
    "Level" : 11,
    "LowpassCutoffFrequency" : 12,
    "LowpassQFactor" : 13,
    "LowpassFilterDistortion" : 14,
    "PeakFrequency" : 15,
    "PeakGain" : 16,
    "PeakQFactor" : 17,
    "Pulsewidth" : 18,
    "PlaybackRate" : 19,
    "PlaybackStart" : 20,
    "PlaybackLoop" : 21,
    "Release" : 22,
    "ReverbDampening" : 23,
    "ReverbMix" : 24,
    "ReverbRoomsize" : 25,
    "SampleBufferNumber" : 26,
    "Samplerate" : 27,
    "StereoPosition" : 28,
    "Sustain" : 29,
});

const sourceParameterShortName = Object.freeze({
    "atk" : 0,
    "dec" : 1,
    "del-damp" : 2,
    "del-fb" : 3,
    "del" : 4,
    "del-time" : 5,
    "dur" : 6,
    "freq" : 7,
    "note" : 8,
    "hp-freq" : 9,
    "hp-q" : 10,
    "lvl" : 11,
    "lp-freq" : 12,
    "lp-q" : 13,
    "lp-dist" : 14,
    "pf-freq" : 15,
    "pf-gain" : 16,
    "pf-q" : 17,
    "pw" : 18,
    "rate" : 19,
    "start" : 20,
    "loop" : 21,
    "rel" : 22,
    "rev-damp" : 23,
    "rev" : 24,
    "rev-rs" : 25,
    "buf" : 26,
    "sr" : 27,
    "pos" : 28,
    "sus" : 29,
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

	    // custom sample loader
	    var customSample = document.getElementById('custom-sample');
	    customSample.addEventListener("change", function() {		
		var sample_id = this.files[0].name.split("\.")[0];		
		var reader = new FileReader();
		reader.onload = function(ev) {
		    ctx.decodeAudioData(ev.target.result)
			.then(r => n.port.postMessage({ type: 'loadSample', samples: r.getChannelData(0), length: r.length, sample_id: sample_id }))
		}
    		reader.readAsArrayBuffer(this.files[0]);
	    }, false);
	    
	    //////////////////////////
	    // LOAD EVENT SCHEDULER //
	    //////////////////////////
	    
	    // this only works in latest chrome/chromium with experimental features flag enabled ...
	    //let scheduler = new Worker('js/scheduler.js', { type : "module"});

	    // set event scheduler message callback
	    //scheduler.onmessage = function(event) {
		// dispatch to sampler module ...
	    //n.port.postMessage({ type: 'trigger', event: event.data })
	    //};

	    run_wasm();

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

	    // master effects - reverb
	    const rev_roomsize = document.getElementById('reverb-roomsize-knob')
	    rev_roomsize.addEventListener('input', e => {
		n.parameters.get('reverb_roomsize').value = e.target.value
	    })
	    
	    const rev_damp = document.getElementById('reverb-dampening-knob')
	    rev_damp.addEventListener('input', e => {
		n.parameters.get('reverb_dampening').value = e.target.value
	    })

	    // master effects - delay
	    const del_time = document.getElementById('delay-time-knob')
	    del_time.addEventListener('input', e => {
		n.parameters.get('delay_time').value = e.target.value
	    })
	    
	    const del_feedback = document.getElementById('delay-feedback-knob')
	    del_feedback.addEventListener('input', e => {
		n.parameters.get('delay_feedback').value = e.target.value
	    })

	    const del_cutoff = document.getElementById('delay-cutoff-knob')
	    del_cutoff.addEventListener('input', e => {
		n.parameters.get('delay_cutoff').value = e.target.value
	    })
	    	    
	    
	    /*
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
	    })*/

	    ///////////////////////////	    
	    // SET KEYBOARD TRIGGERS //
	    ///////////////////////////

	    /*
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
	    })  */

	});
}
