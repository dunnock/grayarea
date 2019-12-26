# 2019-12-25 Test 1

End to end single thread throughput benchmark following the route on a single PC (Mac Book '14):

`WASM sender through runtime -> IPC router -> runtime to WASM receiver`

## 1_000B messages -> 241 Mbit per second

2019-12-25T22:12:55.848 INFO  send             > Sent 100001 messages in 3163 ms
2019-12-25T22:12:55.848 INFO  send             > Message size 1000 speed 241 mbps
2019-12-25T22:12:55.882 INFO  receive          > Processed 100001 messages in 3228 ms

## 100_000B messages -> 608 Mbit per second

2019-12-25T22:13:36.359 INFO  send             > Sent 10001 messages in 12528 ms
2019-12-25T22:13:36.359 INFO  send             > Message size 100000 speed 608 mbps
2019-12-25T22:13:37.838 INFO  receive          > Processed 10001 messages in 14010 ms

## 1_000_000B messages -> 756 Mbit per second

2019-12-25T22:18:27.015 INFO  send             > Sent 1001 messages in 10082 ms
2019-12-25T22:18:27.015 INFO  send             > Message size 1000000 speed 756 mbps
2019-12-25T22:18:31.744 INFO  receive          > Processed 1001 messages in 14850 ms

## Conclusion

Somewhere there is too much memory copying or message handling, probably on a router side