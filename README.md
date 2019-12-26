No docs yet

# How to run:

### Install runtime:
```
cargo install --path=grayarea-runtime --force
```
(make sure cargo/bin is in PATH)

### Build one of examples:

```
cargo wasi build --package=throughput --release
```

### Start desktop engine with given settings:

```
RUST_LOG=info cargo run --release --package=grayarea-desktop examples/throughput/functions.yml
```