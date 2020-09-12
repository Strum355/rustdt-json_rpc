// Copyright 2016 Bruno Medeiros
//
// Licensed under the Apache License, Version 2.0 
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>. 
// This file may not be copied, modified, or distributed
// except according to those terms.


extern crate serde_json;
extern crate serde;

use serde_json::Value;

use jsonrpc_request::*;
use jsonrpc_response::*;
use json_util::*;

/* -----------------  Message  ----------------- */

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Request(Request),
    Response(Response),
}

impl From<Response> for Message {
    fn from(response: Response) -> Self {
        Message::Response(response)
    }
}

impl From<Request> for Message {
    fn from(request: Request) -> Self {
        Message::Request(request)
    }
}

impl serde::Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        match *self {
            Message::Request(ref request) => request.serialize(serializer),
            Message::Response(ref response) => response.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Message {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
        where DE: serde::Deserializer<'de> 
    {
        let mut helper = SerdeJsonDeserializerHelper::new(&deserializer);
        let value = Value::deserialize(deserializer)?;
        let json_obj = helper.as_Object(value)?;
        
        if json_obj.contains_key("method") {
            let request = serde_json::from_value::<Request>(Value::Object(json_obj));
            Ok(Message::Request(request.map_err(to_de_error)?))
        } else {
            let response = serde_json::from_value::<Response>(Value::Object(json_obj));
            Ok(Message::Response(response.map_err(to_de_error)?))
        }
    }
}


#[cfg(test)]
pub mod message_tests {
    
    use super::*;
    use jsonrpc_common::*;
    
    use json_util::test_util::*;
    
    use jsonrpc_response::response_tests::sample_json_obj;
    
    #[test]
    fn test_Message() {
        
        // Attempt Method parse
        test_error_de::<Message>(r#"{ "jsonrpc": "2.0", "method":"foo" }"#, "Property `params` is missing");
        
        // Attempt Response parse
        test_error_de::<Message>(r#"{ "jsonrpc": "2.0"}"#, "Property `id` is missing");
        
        test_serde::<Message>(&Response::new_result(Id::Null, sample_json_obj(100)).into());
        
        let sample_params = unwrap_object(sample_json_obj(123));
        test_serde::<Message>(&Request::new(1, "myMethod".to_string(), sample_params).into());
    }
    
}
