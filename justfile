# record a profile
profile:
    cargo build --profile profiling
    samply record ./target/profiling/scamper run conf.toml
