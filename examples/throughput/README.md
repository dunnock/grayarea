# Build and run:

```
cargo +nightly wasi build --package=throughput --release
cargo install --path=grayarea-runtime --force                                                 
RUST_LOG=info cargo run --release --package grayarea-desktop examples/throughput/functions.yml
```

# Tests

Each test configured by number of messages and size of message in `send.yml`, and will calculate time and throughput.

- `functions.yml` - generate, send and receive random message - final message will cause panic in receiver
- `functions_chk.yml` - send random message, receive and validate checksum for every message - last message will fail check
- `functions_concur.yml` - send random message, 2 recipients listen to same topic will compete - final message will cause panic in receiver or checksum whoever gets it

# Test results

End to end single thread throughput benchmark following the route on a single PC (Mac Book '14):

`WASM sender through runtime -> IPC router -> runtime to WASM receiver`

## 1_000B messages -> 187 MiB/s

2020-01-13T00:20:10.374 INFO  send                    > Sent 1000001 messages in 4408 ms
2020-01-13T00:20:10.374 INFO  send                    > Message size 1000 speed 187 MiB/s
2020-01-13T00:20:10.377 INFO  receive                 > Processed 1000001 messages in 4414 ms

## 10_000B messages -> 986 MiB/s

2020-01-13T00:19:46.203 INFO  send                    > Sent 1000001 messages in 7933 ms
2020-01-13T00:19:46.203 INFO  send                    > Message size 10000 speed 986 MiB/s
2020-01-13T00:19:46.203 INFO  receive                 > Processed 1000001 messages in 7936 ms

## 100_000B messages -> 431 MiB/s

2020-01-13T00:23:40.622 INFO  send                    > Sent 1000001 messages in 200558 ms
2020-01-13T00:23:40.622 INFO  send                    > Message size 100000 speed 431 MiB/s
2020-01-13T00:23:40.764 INFO  receive                 > Processed 1000001 messages in 200709 ms

## Conclusion

Speed is quite good, though is dependent on a message size. 
With average message sizes around 10Kb speed is 2x faster than with 100Kb messages and 5x faster than with 1Kb. 
It might be related with IPC buffer size as well as number of messages throughput.