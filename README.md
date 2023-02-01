# Fluvio Streaming data processing with WasmEdge 

## What is Fluvio?
<hr/>

Fluvio is an open-source data streaming platform with in-line computation capabilities. It utilizes Wasm to support user-defined compute functions in the data stream. 

## Aim-
<hr/>
To integrate WasmEdge as an alternative runtime for Fluvio.

## Run project-
The repo already contains filter wasm files. 

```rust
    cargo build 
```

This repository is created as a solution to pretest [LFX Mentorship 2023 01-Mar-May Challenge - for #2231 #2232](https://github.com/WasmEdge/WasmEdge/discussions/2232)