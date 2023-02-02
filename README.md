# Fluvio Streaming data processing with WasmEdge 

## What is Fluvio?

Fluvio is an open-source data streaming platform with in-line computation capabilities. It utilizes Wasm to support user-defined compute functions in the data stream. 

## Aim-
To integrate WasmEdge as an alternative runtime for [FluvioSmartEngine](https://github.com/infinyon/fluvio/tree/master/crates/fluvio-smartengine) crate.

## About Wasm file-

For test, using the fluvio_smartmodule_filter.wasm file. The function filters the data if input text contains `a` in it. <br>
[Code](https://github.com/infinyon/fluvio/blob/master/smartmodule/examples/filter/src/lib.rs) before compiling the filter function to wasm file.



## Run project-
The repo already contains filter wasm files. 

```rust
    cargo run fluvio_smartmodule_filter 
```

The filter value can be changed by editing array at [Line](https://github.com/Hrushi20/fluvio-smart-engine-wasmedge/blob/main/src/main.rs#L65).
<br>
Output when Filter value is apple: 

``` rust
    let input = SmartModuleInput::try_from(vec![Record::new("apple")])?;           // Line 65 main.rs
```
(Pic)

<br>
Output when Filter value is hello world:

``` rust
     let input = SmartModuleInput::try_from(vec![Record::new("hello world")])?;    // Line 65 main.rs 
```

(Pic)


This repository is created as a solution to pretest [LFX Mentorship 2023 01-Mar-May Challenge - for #2231 #2232](https://github.com/WasmEdge/WasmEdge/discussions/2232)
