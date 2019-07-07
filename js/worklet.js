class RuffboxProcessor extends AudioWorkletProcessor {
    static get parameterDescriptors() {
	return []
    }

    loadSample(sampleData, sampleSize, id){
	
	if(!this._sampleBuffers){
	    this._sampleBuffers = [];
	}
	
	let samplePtr = this._wasm.exports.alloc()	
	let sampleBuf = new Float32Array (
	    this._wasm.exports.memory.buffer,
	    samplePtr,
	    sampleSize
	)	
	
	// copy to wasm buffer 
	sampleBuf.set(sampleData);
	//console.log("LOADED size: " + sampleSize + " -- data: " + sampleData );
	let bufNum = this._wasm.exports.load(samplePtr, sampleSize);

	if(!this._sampleMapping) {
	    this._sampleMapping = {};
	}

	this._sampleMapping[id] = bufNum;
		
	this._sampleBuffers.push([samplePtr, sampleBuf, sampleSize]);	
    }
        
    constructor(options) {
	super(options)
		
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
	    } else if (e.data.type === 'trigger') {
		if(this._wasm) {		    		    
		    this._wasm.exports.trigger(this._sampleMapping[e.data.sample_id], e.data.timestamp);
		}
	    }
	}
    }
    
    process(inputs, outputs, parameters) {
	if (!this._wasm) {
	    return true
	}
	
	let output = outputs[0];

	this._wasm.exports.process(this._outPtr_l, this._outPtr_r, this._size, currentTime);
	output[0].set(this._outBuf_l)
	output[1].set(this._outBuf_r)

	return true
    }
}

registerProcessor('ruffbox-processor', RuffboxProcessor)
