# 2019-12-25 Test 1

End to end single thread throughput benchmark following the route on a single PC (Mac Book '14):

`WASM sender through runtime -> IPC router -> runtime to WASM receiver`

## 1_000B messages -> 1495 MiB/s

2020-01-12T21:35:47.423 INFO  send         > Sent 1000001 messages in 5102 ms
2020-01-12T21:35:47.423 INFO  send         > Message size 1000 speed 1495 MiB/s
INFO  receive      > Processed 1000001 messages in 5119 ms

## 10_000B messages -> 7106 MiB/s

2020-01-12T21:29:32.546 INFO  send         > Sent 1000001 messages in 10736 ms
2020-01-12T21:29:32.546 INFO  send         > Message size 10000 speed 7106 MiB/s
INFO  receive      > Processed 1000001 messages in 10750 ms

## 100_000B messages -> 3573 MiB/s

2020-01-12T21:33:48.122 INFO  send         > Sent 1000001 messages in 213514 ms
2020-01-12T21:33:48.122 INFO  send         > Message size 100000 speed 3573 MiB/s
2020-01-12T21:33:48.278 INFO  receive      > Processed 1000001 messages in 213797 ms

## Conclusion

Speed is quite good, though is dependent on a message size. 
With average message sizes around 10Kb speed is 2x faster than with 100Kb messages and 5x faster than with 1Kb. 
It might be related with IPC buffer size as well as number of messages throughput.