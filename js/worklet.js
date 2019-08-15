class RuffboxProcessor extends AudioWorkletProcessor {
    static get parameterDescriptors() {	
	return [
	    {
		name: 'reverb_roomsize',
		defaultValue: 0.65,
	    },
	    {
		name: 'reverb_dampening',
		defaultValue: 0.43,
	    },
	    {
		name: 'delay_time',
		defaultValue: 0.256,
	    },
	    {
		name: 'delay_feedback',
		defaultValue: 0.5,
	    },
	    {
		name: 'delay_cutoff',
		defaultValue: 3000.0,
	    },
	]
    }
   
    loadSample(sampleData, sampleSize, id){
	
	if(!this._sampleBuffers){
	    this._sampleBuffers = [];
	}

	let sampleSizeForInterpolation = sampleSize + 3;
	
	let samplePtr = this._wasm.exports.alloc(sampleSizeForInterpolation);	
	let sampleBuf = new Float32Array (
	    this._wasm.exports.memory.buffer,
	    samplePtr,
	    sampleSizeForInterpolation // to facilitate interpolation
	)	
	
	// copy to wasm buffer, offset one for interpolation
	sampleBuf.set(sampleData, 1);
	//console.log("LOADED size: " + sampleSize + " -- data: " + sampleData );
	let bufNum = this._wasm.exports.load(samplePtr, sampleSizeForInterpolation);

	if(!this._sampleMapping) {
	    this._sampleMapping = {};
	}

	this._sampleMapping[id] = bufNum;
		
	this._sampleBuffers.push([samplePtr, sampleBuf, sampleSizeForInterpolation]);	
    }
        
    constructor(options) {
	super(options)

	this._last_reverb_roomsize = 0.65;
	this._last_reverb_dampening = 0.43;
	this._last_delay_time = 0.256;
	this._last_delay_feedback = 0.5;
	this._last_delay_cutoff = 3000.0;
		
	this.port.onmessage = e => {
	    // unfortunately, this seems to be the only way to load
	    // the wasm module in the worklet.
	    // we have to fetch it here because the worklet scope doesn't expose
	    // 'fetch()'
	    if (e.data.type === 'loadWasm') {
		WebAssembly.instantiate(e.data.data).then(w => {		    
		    this._wasm = w.instance
		    // grow memory to accomodate full sample ... 
		    this._wasm.exports.memory.grow(250)
		    this._size = 128
		    		    
		    if(this._samples) {
			this._samples.forEach(
			    function(sampleInfo) {
				this.loadSample(sampleInfo[0], sampleInfo[1], sampleInfo[2]);
			    }, this);
			this._samples = [];			
		    }

		    // why always last ??
		    this._outPtr_r = this._wasm.exports.alloc(this._size)		    
		    this._outBuf_r = new Float32Array (
			this._wasm.exports.memory.buffer,
			this._outPtr_r,
			this._size
		    )
		    this._outPtr_l = this._wasm.exports.alloc(this._size)		    
		    this._outBuf_l = new Float32Array (
			this._wasm.exports.memory.buffer,
			this._outPtr_l,
			this._size
		    )
		})		
	    } else if (e.data.type === 'loadSample') {
				
		let sampleSize = e.data.length;
		let sampleData = e.data.samples;
		let sampleId = e.data.sample_id;
		
		if(!this._samples){
		    this._samples = [];
		}
		
		if(this._wasm){
		    loadSample(sampleData, sampleSize, sampleId);
		} else {
		    this._samples.push([sampleData, sampleSize, sampleId]);
		}
	    } else if (e.data.type === 'sourceType') {
		this._sourceType = e.data.content;
	    } else if (e.data.type === 'sourceParameter') {
		this._sourceParameter = e.data.content;
	    } else if (e.data.type === 'trigger') {
		if(this._wasm) {
		    let event = e.data.event;
		    let params = e.data.event.params;
		    console.log(event);
		    let instance_id = this._wasm.exports.prepare(this._sourceType[event.source_type], event.timestamp, this._sampleMapping[event.sample_id]);
		    for (let [key, value] of Object.entries(params)) {
		    	this._wasm.exports.set_instance_parameter(instance_id, this._sourceParameter[key], value);
		    }
		    this._wasm.exports.trigger(instance_id);
		}
	    }
	}

	if(!this._sampleMapping) {
	    this._sampleMapping = {};
	}
    }
    
    process(inputs, outputs, parameters) {
	if (!this._wasm) {
	    return true
	}
	
	let output = outputs[0];

	if(this._last_reverb_roomsize != parameters.reverb_roomsize[0]) {
	    this._wasm.exports.set_master_parameter(25, parameters.reverb_roomsize[0]);
	    this._last_reverb_roomsize = parameters.reverb_roomsize[0];
	}

	if(this._last_reverb_dampening != parameters.reverb_dampening[0]) {
	    this._wasm.exports.set_master_parameter(23, parameters.reverb_dampening[0]);
	    this._last_reverb_dampening = parameters.reverb_dampening[0];
	}

	if(this._last_delay_time != parameters.delay_time[0]) {
	    this._wasm.exports.set_master_parameter(5, parameters.delay_time[0]);
	    this._last_delay_time = parameters.delay_time[0];
	}

	if(this._last_delay_feedback != parameters.delay_feedback[0]) {
	    this._wasm.exports.set_master_parameter(3, parameters.delay_feedback[0]);
	    this._last_delay_feedback = parameters.delay_feedback[0];
	}

	if(this._last_delay_cutoff != parameters.delay_cutoff[0]) {
	    this._wasm.exports.set_master_parameter(2, parameters.delay_cutoff[0]);
	    this._last_delay_cutoff = parameters.delay_cutoff[0];
	}
	
	this._wasm.exports.process(this._outPtr_l, this._outPtr_r, this._size, currentTime);
	output[0].set(this._outBuf_l)
	output[1].set(this._outBuf_r)

	return true
    }
}

registerProcessor('ruffbox-processor', RuffboxProcessor)
