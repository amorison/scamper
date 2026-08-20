# record a profile
profile:
    cargo build --profile profiling
    samply record ./target/profiling/lpm run conf.toml
