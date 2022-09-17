```rust
        .arg(
            Arg::new("parameters")
                .short('p')
                .long("parameters")
                .help("Extra parameters of used compressor"),
        )
        .arg(
            Arg::new("compressor")
                .long("compress-cmd")
                .help("External compressor command line. This overrides the default method"),
        )
```