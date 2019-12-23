## grayarea format reference

Unique name of a function including the version:
```
name: "polo-websocket:1.0"
```

Specify path/registry link to the module:
```
module:
  path: "target/wasm32-wasi/release/subscriber.wasm"
```

Command line arguments to initialize main function:
```
args: ["USDT_BTC"]
```

Subscription module shall specify resource and connection string. It might also provide custom subscription logic in supplied wasm main() function
```
stream:
  websocket:
    url: "wss://api2.poloniex.com:443"
```