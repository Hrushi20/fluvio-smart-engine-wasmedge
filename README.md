# Fluvio Streaming data processing with WasmEdge 

## What is Fluvio?

Fluvio is an open-source data streaming platform with in-line computation capabilities. It utilizes Wasm to support user-defined compute functions in the data stream. 

## Aim-
To integrate WasmEdge as an alternative runtime for [FluvioSmartEngine](https://github.com/infinyon/fluvio/tree/master/crates/fluvio-smartengine) crate.

## Current Status- 
* While executing the wasm function, error being thrown (invalid memory reference) [More](https://github.com/WasmEdge/WasmEdge/discussions/2232#discussioncomment-4832922)
* Copy/Clone trait is not present on [Memory](https://wasmedge.github.io/WasmEdge/wasmedge_sdk/struct.Memory.html). Therefore, can't extract the output of the execute wasm function. Hence, commented the [code](https://github.com/Hrushi20/fluvio-smart-engine-wasmedge/blob/main/src/transforms/filter.rs#L126) to verify the ouput in tests.

## Run project-
The repo already contains filter wasm files. 

```rust
    cargo test
```

This repository is created as a solution to pretest [LFX Mentorship 2023 01-Mar-May Challenge - for #2231 #2232](https://github.com/WasmEdge/WasmEdge/discussions/2232)
