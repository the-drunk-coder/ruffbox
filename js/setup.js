const sourceType = Object.freeze({
    "Sampler" : 0,    
    "LiveSampler" : 1,
    "FrozenSampler" : 2,
    "SineOsc" : 3,
    "SineSynth": 4 ,
    "LFCubSynth": 5,
    "LFSawSynth": 6,
    "LFSquareSynth": 7,
    "LFTriangleSynth": 8,
    "RissetBell": 9,
    "Wavetable": 10,
    "Wavematrix": 11,
});

const sourceParameter = Object.freeze({
    "Attack":                   0,
    "Decay":                    1,
    "DelayDampeningFrequency":  2,
    "DelayFeedback":            3,
    "DelayMix":                 4,
    "DelayTime":                5,
    "DelayRate":                6,
    "Duration":                 7,
    "PitchFrequency":           8,
    "PitchNote":                9,
    "HighpassCutoffFrequency":  10,
    "HighpassQFactor":          11,
    "EnvelopeLevel":            12,
    "OscillatorAmplitude":      13,// (oscillator amplitude)
    "OscillatorPhaseRelative":  14,// (radians)
    "OscillatorPhaseEffective": 15,// (the target value or effective offset dependent on the amplitude)
    "LowpassCutoffFrequency":   16,
    "LowpassQFactor":           17,
    "LowpassFilterDistortion":  18,
    "PeakFrequency":            19,
    "PeakGain":                 20,
    "PeakQFactor":              21,
    "Pulsewidth":               22,
    "PlaybackRate":             23,
    "PlaybackStart":            24,
    "PlaybackLoop":             25,
    "Release":                  26,
    "ReverbDampening":          27,
    "ReverbMix":                28,
    "ReverbRoomsize":           29,
    "SampleBufferNumber":       30,
    "Samplerate":               31,
    "ChannelPosition":          32,
    "AmbisonicAzimuth":         33,
    "AmbisonicElevation":       34,
    "Sustain":                  35,
    "Wavetable":                36,
    "Wavematrix":               37,
    "WavematrixTableIndex":     38,
});

const sourceParameterShortName = Object.freeze({
    "atk" :      0,
    "dec" :      1,
    "del-damp" : 2,
    "del-fb" :   3,
    "del" :      4,
    "del-time" : 5,
    "del-rate" : 6,
    "dur" :      7,
    "freq" :     8,
    "note" :     9,
    "hpf" :      10,
    "hpq" :      11,
    "lvl" :      12,
    "amp" :      13,
    "phase" :    14,
    "eff_phase": 15,
    "lpf" :      16,
    "lpq" :      17, 
    "lpd" :      18,
    "pff" :      19,
    "pfg" :      20,
    "pfq" :      21,
    "pw" :       22,
    "rate" :     23,
    "start" :    24,
    "loop" :     25,
    "rel" :      26,
    "rev-damp" : 27,
    "rev" :      28,
    "rev-rs" :   29,
    "buf" :      30,
    "sr" :       31,
    "pos" :      32,
    "azi" :      33,
    "ele" :      34,
    "sus" :      35,
    "wt" :       36,
    "wm" :       37,
    "wti" :      38,
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
	    fetch('wasm/ruffbox_synth_web.wasm?t=' + new Date().getTime())
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
	    let scheduler = new Worker('js/scheduler.js');

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
	    	    
	    // scheduler controls
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
