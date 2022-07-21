# ruffbox

A simple sampler/synth with text-based step sequencer that runs in the browser.

Currently needs Chromium/Chrome 74+ for the *AudioWorklet*, and the *#enable-experimental-web-platform-features* flag enabled for the module workers.

## Project Structure

* **ruffbox-synth** contains synthesizer part of the application
* **ruffbox-pattern** contains the Rust part of the text-based step-sequencer/scheduler
* **js/scheduler.js** contains the JavaScript WebWorker part of the scheduler/pattern sequencer
* **js/setup.js** contains the JavaScript glue code to put things together, load the audio context etc.
* **js/worklet.js** contains the AudioWorkletProcessor.

## Running Locally
* Make sure you have Python3 for the http server!
* Checkout repository.
* Run `build.sh`.
* Run `server.sh`.
* Open your browser (Opera or Chrome) and go to https://localhost:1234. 

The script displays https://0.0.0.0:1234, but that won't work because it's not recognized as local. Use localhost!

## Basic Idea

This basically is (or is evolving from) a proof-of-concept on how to make a multi-threaded audio application that runs in a web browser.

It has three parts that communicate over message-passing:

* the *ruffbox-synth*, a synth player that runs inside an *AudioWorkletProcessor*
* the *ruffbox-pattern*, a sequence generator that runs in a *Web Worker* thread
* the user interaction part, running in the main thread

### Challenges & Development History

Starting to explore the possibilities of Rust and WebAssembly for web audio, I first of all stumbled over this project:
https://github.com/reprimande/wasm-audioworklet-synth. 

It gives the basic outline on how to load a WASM module in an AudioWorkletProcessor. The author wrote an article about it:
https://qiita.com/reprimande/items/5c078e5a7f9f52d2091c. I can't read Japanese, but the code is clear enough, luckily.

One main observation: as of 2019, it's not possible to load a module inside the WorkletProcessor directly
so you have to send the raw binary to the Worklet using its message port. 

The synth project is lacking two things, though. I'm more of a sample user than a synth user, so in the long term, sample
playback would be nice. The synth project was also lacking precise scheduling, which is indispensable when making any kind
of rhythmic music. 

The basic sample playback part was relatively easy: https://github.com/the-drunk-coder/wasm-loop-player. Samples can be loaded 
the same way as the WASM module, fetching them in the main thread and posting them to the audio worklet using the message port.
A challenge here was to provide reliable buffer allocation. Somehow the buffers get neutered easily.

Observation: the order of allocation seems important. First, allocate the sample buffers, then allocate the output buffers.

The next challenge was the scheduling part. The original idea was to write as much code as possible in Rust, but as of 2019, 
WASM threads aren't available yet, so I didn't get around using a web worker thread. The experimental module web worker allows you to
import a WASM module (this time generated using `wasm-bindgen`) directly in the worker (see [scheduler.js](js/scheduler.js) ).

As of 2022, it seems that threads still are somewhat tricky to use (`wasm_thread` is ready but doesn't allow to use `postMessage`), 
and `stdweb` seems broken and hasn't been maintained for a while, so I moved the time iteration to plain JS, and only use the Rust 
part for parsing and generating the sequences, as well as doing the time compensation, only using the methods exposed by `wasm_bindgen`
and a custom data structure.
