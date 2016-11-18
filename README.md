# RustDT JSON-RPC

A JSON-RPC library for Rust. 

Originally created because I wanted to developed a "real-world" project to effectively learn Rust 
(especially with code related to concurrency/multi-threading).
Also, at the time, [jsonrpc-core](https://github.com/ethcore/jsonrpc-core) didn't support asynchronous
method handling.

### As compared to [jsonrpc-core](https://github.com/ethcore/jsonrpc-core)

 * Supports both client and server directions (The same endpoint can be used for both). jsonrpc-core only server handling, currently.
 * Does't support batch requests, jsonrpc-core does.
 * Some minor implementations details: TODO describe more?
   * id support? Must be a JSON String, Null, or Number fitting into a unsigned 64 bits integer. 

### Projects using rustdt_json_rpc:
 * [RustLSP](https://github.com/RustDT/RustLSP)
