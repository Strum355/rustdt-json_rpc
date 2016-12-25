// Copyright 2016 Bruno Medeiros
//
// Licensed under the Apache License, Version 2.0 
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>. 
// This file may not be copied, modified, or distributed
// except according to those terms.

/*!

JSON-RPC Server and client full example.

This example creates a JSON-RPC server listening on a TCP socket,
then a client connecting to the server,
and then has the client invoke method `my_method` and await the result.

*/

extern crate jsonrpc;
extern crate futures;
extern crate serde;

#[macro_use] extern crate log;
extern crate env_logger;

mod tests_sample_types;

use jsonrpc::method_types::MethodResult;
use jsonrpc::EndpointHandler;
use jsonrpc::Endpoint;
use jsonrpc::RequestFuture;
use jsonrpc::NullRequestHandler;
use jsonrpc::map_request_handler::MapRequestHandler;
use jsonrpc::output_agent::OutputAgent;
use jsonrpc::service_util::{WriteLineMessageWriter, ReadLineMessageReader};

use std::thread;
use std::net::{TcpStream, TcpListener};
use std::io::BufReader;
use futures::Future;

use tests_sample_types::Point;

use log::LogLevelFilter;
use env_logger::LogBuilder;


fn my_method(params: Point) -> MethodResult<String, ()> {
    Ok(format!("Got params: x: {}, y: {}.", params.x, params.y))
}

#[test]
pub fn test_client_server_communication() {
    init_logger(LogLevelFilter::Info);
    
    info!("Running example...");

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let local_addr = listener.local_addr().unwrap();
    
    // Spawn thread to handle server requests
    thread::spawn(|| {
        run_server_listener(listener)
    });
    
    // Now prepare client connection
    let stream = TcpStream::connect(local_addr).unwrap();
    
    let msg_writer = WriteLineMessageWriter(stream.try_clone().expect("Failed to clone stream"));
    let output_agent = OutputAgent::start_with_provider(|| msg_writer);
    let mut endpoint = Endpoint::start_with(output_agent);
    
    let endpoint2 = endpoint.clone();
    // Create a thread to handle the client endpoint
    thread::spawn(|| {
        // Note that client endpoint can act as a server too, it can also serve requests.
        // But in this example request_handler is set up to error on any request.
        let request_handler = NullRequestHandler{};
        let endpoint = EndpointHandler::create(endpoint2, Box::new(request_handler));
        let mut msg_reader = ReadLineMessageReader(BufReader::new(stream));
        endpoint.run_message_read_loop(&mut msg_reader).ok();
    });
    
    let params = Point{ x: 10, y: 20};
    // Send the RPC request.
    // Note serde_json deserialize/serialize will be applied to params:
    let future = endpoint.send_request("my_method", params);
    let future : RequestFuture<String, ()> = future.expect("Failed to send RPC request to for `my_method`.");
    
    let response_result = future.wait().unwrap();
    let result : String = response_result.unwrap_result().unwrap();
    assert_eq!(result, "Got params: x: 10, y: 20.".to_string());
    
    // shutdown endpoint
    endpoint.shutdown_and_join();
}

fn run_server_listener(listener: TcpListener) {
    for stream in listener.incoming() {
        let stream = stream.expect("TCP listen error.");
        thread::spawn(move|| handle_server_connection(stream));
        
        break; // For example purposes, we only listen to first connection
    }
    drop(listener);
}


fn handle_server_connection(stream: TcpStream) {
    let mut request_handler = MapRequestHandler::new();
    request_handler.add_request("my_method", Box::new(my_method));
    
    let msg_writer = WriteLineMessageWriter(stream.try_clone().expect("Failed to clone stream"));
    let endpoint = EndpointHandler::create_with_writer(msg_writer, Box::new(request_handler));
    
    let mut msg_reader = ReadLineMessageReader(BufReader::new(stream));
    endpoint.run_message_read_loop(&mut msg_reader).ok();
}

fn init_logger(level: LogLevelFilter) {
    // Prepare log, set info as default log level 
    let mut builder = LogBuilder::new();
    builder.filter(None, level);
    
    if let Ok(rustlog_env_var) = std::env::var("RUST_LOG") {
        builder.parse(&rustlog_env_var);
    }
    builder.init().unwrap();
}