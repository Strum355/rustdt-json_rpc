// sample serde types, taken from serde doc: 
// https://github.com/serde-rs/serde#deserialization-without-macros

#![cfg(test)]

use std::fmt;

use serde;
use serde::ser::SerializeStruct;

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn new_sample_params(x: i32, y: i32) -> Point {
    Point { x : x, y : y }
}

#[test]
fn test_Point() {
    use json_util::test_util::*;
    
    test_serde(&Point{ x: 12, y : 34});
}

pub enum PointField {
    X,
    Y,
}


impl<'de> serde::Deserialize<'de> for PointField {
    fn deserialize<D>(deserializer: D) -> Result<PointField, D::Error>
        where D: serde::de::Deserializer<'de>
    {
        struct PointFieldVisitor;

        impl<'de> serde::de::Visitor<'de> for PointFieldVisitor {
            type Value = PointField;

            fn visit_str<E>(self, value: &str) -> Result<PointField, E>
                where E: serde::de::Error
            {
                match value {
                    "x" => Ok(PointField::X),
                    "y" => Ok(PointField::Y),
                    _ => Err(serde::de::Error::custom("expected x or y")),
                }
            }

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
            {
                formatter.write_str("x or y")
            }
        }

        deserializer.deserialize_any(PointFieldVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Point, D::Error>
        where D: serde::de::Deserializer<'de>
    {
        static FIELDS: &'static [&'static str] = &["x", "y"];
        deserializer.deserialize_struct("Point", FIELDS, PointVisitor)
    }
}

struct PointVisitor;

impl<'de> serde::de::Visitor<'de> for PointVisitor {
    type Value = Point;

    fn visit_map<V>(self, mut visitor: V) -> Result<Point, V::Error>
        where V: serde::de::MapAccess<'de>
    {
        let mut x = None;
        let mut y = None;

        loop {
            match visitor.next_key()? {
                Some(PointField::X) => { x = Some(visitor.next_value())?; }
                Some(PointField::Y) => { y = Some(visitor.next_value())?; }
                None => { break; }
            }
        }

        let x = match x {
            Some(x) => x,
            None => return Err(<V::Error as serde::de::Error>::missing_field("x")),
        };

        let y = match y {
            Some(y) => y,
            None => return Err(<V::Error as serde::de::Error>::missing_field("y")),
        };

        Ok(Point{ x: x, y: y })
    }

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
    {
        formatter.write_str("a point")
    }
}


impl serde::Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        let elem_count = 2;
        let mut state = serializer.serialize_struct("Point", elem_count)?; 
        {
            state.serialize_field("x", &self.x)?;
            state.serialize_field("y", &self.y)?;
        }
        state.end()
    }
}
