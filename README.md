# ruffbox

A simple, text-based step sequencer that runs in the browser.

Currently needs Chromium/Chrome 74+ for the *AudioWorklet*, and the *#enable-experimental-web-platform-features* flag enabled for the module workers.

## Basic Idea

This is basically a proof-of-concept on how to make a multi-threaded audio application that runs in a web browser.

It has three parts that communicate over message-passing:

* the *ruffbox-sampler*, a sample player that runs inside an *AudioWorkletProcessor*
* the *ruffbox-scheduler*, a seqeunce generator that runs in a *Web Worker* thread
* the user interaction part, running in the main thread
