# PoolBoy

Poolboy is intended to sit between a group of localized crypto miners and a mining pool.

The end-goal is to be able to connect to multiple pools, and work for PPS-based pools during the network RTT between shares from your preferred pool.

This code is not complete, nor near finished.


```
cargo run -- --pool eth-us-west.flexpool.io:4444 --wallet 0x000000000000000000000000000000000000dEaD --worker foobar
```

