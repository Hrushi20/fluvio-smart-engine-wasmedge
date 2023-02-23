# Fluvio Streaming data processing with WasmEdge 

## What is Fluvio?

Fluvio is an open-source data streaming platform with in-line computation capabilities. It utilizes Wasm to support user-defined compute functions in the data stream. 

## Aim-
To integrate WasmEdge as an alternative runtime for [FluvioSmartEngine](https://github.com/infinyon/fluvio/tree/master/crates/fluvio-smartengine) crate.

## About Wasm file-

* For test, using the fluvio_smartmodule_filter.wasm file. 
* The function filters the data if input text contains `a` in it i.e it accepts words containing 'a' in it and rejects remaning words.
* [Code](https://github.com/infinyon/fluvio/blob/master/smartmodule/examples/filter/src/lib.rs) before compiling the filter function to wasm file.
* The above code file can be converted into wasm file by running `make` command in `fluvio/smartmodule/examples` [directory](https://github.com/infinyon/fluvio/tree/master/smartmodule/examples) in fluvio repository



## Run project-
The repo already contains filter wasm files. 

Output when input is apple: 
```rust
    cargo run fluvio_smartmodule_filter apple  
```

![Apple](./apple.png)

<br>

Output when input is hello world:
```rust
    cargo run fluvio_smartmodule_filter hello-world  
```

![Hello World](./hello-world.png)

#### Note-
Multiple inputs can be passed to the wasm function through cli using- 
```rust
    cargo run fluvio_smartmodule_filter hello-world apple banana
```

## Tests- 
Added tests to the code.  
```rust
    cargo test 
```

Output of test
```
test test::test_filter_with_incorrect_input ... ok
test test::test_filter_with_correct_input ... ok
test test::test_filter_with_mixed_input ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```


## Statistics- 
The code is configured to generate statistics during execution of wasm functions in wasmedge. 

``` rust

    let stats_config = StatisticsConfigOptions::new().count_instructions(true).measure_cost(true).measure_time(true);
   
    let config = ConfigBuilder::with_statistics_config(ConfigBuilder::default(),stats_config).build()?;

    let mut stats = Statistics::new().expect("Unable to init statistics struct");

    // create an executor
    let mut executor = Executor::new(Some(&config), Some(&mut stats))?;
```

This repository is created as a solution to pretest [LFX Mentorship 2023 01-Mar-May Challenge - for #2231 #2232](https://github.com/WasmEdge/WasmEdge/discussions/2232)
