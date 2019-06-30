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

	    if(ctx.state === "suspended") {
		ctx.resume();
	    }

	    window.onkeyup = function(e) {
		var key = e.keyCode ? e.keyCode : e.which;
		
		if (key === 66) {
		    n.port.postMessage({ type: 'trigger', sample_id: 'bd'})
		} else if (key === 83) {
		    n.port.postMessage({ type: 'trigger', sample_id: 'sn'})
		}
	    }
	    
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
	    	    	    
	    fetch('wasm/ruffbox.wasm?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => n.port.postMessage({ type: 'loadWasm', data: r }))

	    fetch('audio/bd.flac?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => ctx.decodeAudioData(r)
		      .then(r => n.port.postMessage({ type: 'loadSample', samples: r.getChannelData(0), length: r.length, sample_id: 'bd' })))

	    fetch('audio/sn.flac?t=' + new Date().getTime())
		.then(r => r.arrayBuffer())
		.then(r => ctx.decodeAudioData(r)
		      .then(r => n.port.postMessage({ type: 'loadSample', samples: r.getChannelData(0), length: r.length, sample_id: 'sn' })))

	    
	})   
}
