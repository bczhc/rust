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

About path strings:

On Unix things are very easy: paths can be in arbitrary binary (meaning no need
to be valid UTF-8 encoded sequences).
So I just use `OsString` with unix-specific "from [u8]"-like functions;

But I'm not familiar with this on Windows. I don't know how Windows handles
these. Now it will panic when meeting invalid UTF-8.

And I know these restrictions are related with filesystems. For example ext4
and Btrfs allow arbitrary binary except '\0' as file names, and I don't know how
Windows will handle this when use some methods to mount these filesystems
on a Windows platform.