// Copyright 2016 Bruno Medeiros
//
// Licensed under the Apache License, Version 2.0 
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>. 
// This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

use serde;
use serde::de::Visitor;
use serde::de;
use serde::ser::SerializeStruct;

use serde_json;
use serde_json::Value;

use util::core::GResult;

use jsonrpc_common::*;
use json_util::*;

/* -----------------  ----------------- */

pub fn check_jsonrpc_field<ERR, HELPER>(helper: &mut HELPER, json_obj: &mut JsonObject) -> Result<(), ERR>
where 
    HELPER: JsonDeserializerHelper<ERR>, 
{
    let jsonrpc = try!(helper.obtain_String(json_obj, "jsonrpc"));
    if jsonrpc != "2.0" {
        return Err(helper.new_error(r#"Property `jsonrpc` is not "2.0". "#))
    };
    Ok(())
}

/* -----------------  Request  ----------------- */

/// A JSON RPC request, version 2.0
#[derive(Debug, PartialEq, Clone)]
pub struct Request {
    // ommited jsonrpc field, must be "2.0" when serialized
    //pub jsonrpc : String, 
    pub id : Option<Id>,
    pub method : String,
    pub params : RequestParams,
}

impl Request {
    pub fn new(id_number: u64, method: String, params: JsonObject) -> Request {
        Request {
            id : Some(Id::Number(id_number)),
            method : method,
            params : RequestParams::Object(params),
        } 
    }
}

impl serde::Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        // TODO: need to investigate if elem_count = 4 is actually valid when id is missing
        // serializing to JSON seems to not be a problem, but there might be other issues
        let elem_count = 4;
        let mut state = try!(serializer.serialize_struct("Request", elem_count)); 
        {
            try!(state.serialize_field("jsonrpc", "2.0"));
            if let Some(ref id) = self.id {
                try!(state.serialize_field("id", id));
            }
            try!(state.serialize_field("method", &self.method));
            try!(state.serialize_field("params", &self.params));
        }
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Request {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
        where DE: serde::Deserializer<'de> 
    {
        let mut helper = SerdeJsonDeserializerHelper::new(&deserializer);
        let value = try!(Value::deserialize(deserializer));
        let mut json_obj = try!(helper.as_Object(value));
        
        try!(check_jsonrpc_field(&mut helper, &mut json_obj));
        
        let id = json_obj.remove("id");
        let id = try!(id.map_or(Ok(None), |value| serde_json::from_value(value).map_err(to_de_error)));
        let method = try!(helper.obtain_String(&mut json_obj, "method"));
        let params = try!(helper.obtain_Value(&mut json_obj, "params"));
        
        let params = try!(to_jsonrpc_params(params).map_err(to_de_error));
        
        Ok(Request { id : id, method : method, params : params })
    }
}


/* -----------------  ----------------- */

#[derive(Debug, PartialEq, Clone)]
pub enum RequestParams {
    Object(JsonObject),
    Array(Vec<Value>),
    None,
}

impl RequestParams {
    pub fn into_value(self) -> Value {
        // Note, we could use serde_json::to_value(&params) but that is less efficient:
        // it reserializes the value, instead of just obtaining the underlying one 
        
        match self {
            RequestParams::Object(object) => Value::Object(object),
            RequestParams::Array(array) => Value::Array(array),
            RequestParams::None => Value::Null,
        }
    }
}

impl serde::Serialize for RequestParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        match *self {
            RequestParams::Object(ref object) => object.serialize(serializer),
            RequestParams::Array(ref array) => array.serialize(serializer),
            RequestParams::None => serializer.serialize_none(),
        }
    }
}

pub fn to_jsonrpc_params(params: Value) -> GResult<RequestParams> {
    match params {
        Value::Object(object) => Ok(RequestParams::Object(object)),
        Value::Array(array) => Ok(RequestParams::Array(array)),
        Value::Null => Ok(RequestParams::None),
        _ => Err("Property `params` not an Object, Array, or null.".into()),
    }
}

impl<'de> serde::Deserialize<'de> for RequestParams {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
        where DE: serde::Deserializer<'de> 
    {
        deserializer.deserialize_any(RequestParams_DeserializeVisitor)
    }
}

struct RequestParams_DeserializeVisitor;

impl<'de> Visitor<'de> for RequestParams_DeserializeVisitor {
    type Value = RequestParams;
    
    fn visit_unit<E>(self) -> Result<Self::Value, E> 
    {
        Ok(RequestParams::None)
    }
    
    fn visit_seq<V>(self, mut access: V) -> Result<Self::Value, V::Error>
        where V: de::SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = access.next_element()? {
            values.push(value);
        }
        Ok(RequestParams::Array(values))
    }

    fn visit_map<V>(self, mut access: V) -> Result<Self::Value, V::Error>
        where V: de::MapAccess<'de>,
    {
        let mut values = serde_json::value::Map::new();
        while let Some((key, value)) = access.next_entry()? {
            values.insert(key, value);
        }
        Ok(RequestParams::Object(values))
    }
    
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
    {
        formatter.write_str("params")
    }
}



#[cfg(test)]
pub mod request_tests {

    use super::*;
    
    use util::tests::*;
    use json_util::*;
    use json_util::test_util::*;
    use jsonrpc_common::*;
    
    use serde_json::Value;


    #[test]
    fn test__RequestParams() {
        
        let sample_obj = json!({"xxx": 123}).as_object().unwrap().clone();
        let sample_string = Value::String("blah".into());
        
        test_serde__RequestParams(RequestParams::Object(sample_obj));
        test_serde__RequestParams(RequestParams::Array(vec![sample_string.clone(), sample_string]));
        test_serde__RequestParams(RequestParams::None);
    }
    
    fn test_serde__RequestParams(params: RequestParams) {
        let params_reser = test_serde(&params).0;
        assert_equal(params_reser, params);
    }
    
    pub fn check_error(result: RequestError, expected: RequestError) {
        assert_starts_with(&result.message, &expected.message);
        assert_eq!(result, RequestError { message : result.message.clone(), .. expected }); 
    }
    
    #[test]
    fn test_Request() {
        
        let sample_params = json!({
            "param": "2.0",
            "foo": 123,
        }).as_object().unwrap().clone();
        
        // Test invalid JSON
        test_error_de::<Request>(
            "{",
            "EOF while"
        );
        
        test_error_de::<Request>(
            "{ }",
            "Property `jsonrpc` is missing.",
        );
        
        test_error_de::<Request>(
            r#"{ "jsonrpc": "1.0" }"#,
            r#"Property `jsonrpc` is not "2.0". "#,
        );
        
        test_error_de::<Request>(
            r#"{ "jsonrpc": "2.0" }"#,
            "Property `method` is missing.",
        );
        test_error_de::<Request>(
            r#"{ "jsonrpc": "2.0", "method":null }"#,
            "Value `null` is not a String.",
        );
        
        test_error_de::<Request>(
            r#"{ "jsonrpc": "2.0", "method":"xxx" }"#,
            "Property `params` is missing.",
        );
        
        // Test valid request with params = null
        assert_equal(
            from_json(r#"{ "jsonrpc": "2.0", "method":"xxx", "params":null }"#),
            Request { id : None, method : "xxx".into(), params : RequestParams::None, } 
        );
        
        // --- Test serialization ---
        
        // basic Request
        let request = Request::new(1, "myMethod".to_string(), sample_params);
        test_serde(&request);
        
        // Test basic Request, no params
        let request = Request { id : None, method : "myMethod".to_string(), params : RequestParams::None, };
        test_serde(&request);
        
        // Test Request with no id
        let sample_array_params = RequestParams::Array(vec![]);
        let request = Request { id : None, method : "myMethod".to_string(), params : sample_array_params, };  
        test_serde(&request);
    }
    
}
