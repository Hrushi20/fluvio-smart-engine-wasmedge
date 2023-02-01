# Fluvio Streaming data processing with WasmEdge 

## What is Fluvio?
<hr/>

Fluvio is an open-source data streaming platform with in-line computation capabilities. It utilizes Wasm to support user-defined compute functions in the data stream. 

## Aim-
<hr/>
To integrate WasmEdge as an alternative runtime for Fluvio.

## Current Status- 
* While executing the wasm function, error being thrown (invalid memory reference) [More](https://github.com/WasmEdge/WasmEdge/discussions/2232#discussioncomment-4832922)
* Copy/Clone trait is not present on [Memory](https://wasmedge.github.io/WasmEdge/wasmedge_sdk/struct.Memory.html). Therefore, can't extract the output of the execute wasm function. Hence, commented the code in the test to check the output of the filter.

## Run project-
The repo already contains filter wasm files. 

```rust
    cargo test
```

This repository is created as a solution to pretest [LFX Mentorship 2023 01-Mar-May Challenge - for #2231 #2232](https://github.com/WasmEdge/WasmEdge/discussions/2232)