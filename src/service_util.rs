// Copyright 2016 Bruno Medeiros
//
// Licensed under the Apache License, Version 2.0 
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>. 
// This file may not be copied, modified, or distributed
// except according to those terms.

use std::result::Result;
use std::io;

pub use util::core::GError;
pub use util::core::GResult;


pub trait MessageReader {
    fn read_next(&mut self) -> Result<String, GError>;
}

/// Read a message by reading lines from a BufRead.
/// This is of use mainly for tests and example code.
pub struct ReadLineMessageReader<T: io::BufRead>(pub T);

impl<T : io::BufRead> MessageReader for ReadLineMessageReader<T> {
    fn read_next(&mut self) -> Result<String, GError> {
        let mut result = String::new();
        self.0.read_line(&mut result)?;
        Ok(result)
    }
}

pub trait MessageWriter {
    fn write_message(&mut self, msg: &str) -> Result<(), GError>;
}

/// Handle a message simply by writing to a io::Write and appending a newline.
/// This is of use mainly for tests and example code.
pub struct WriteLineMessageWriter<T: io::Write>(pub T);

impl<T : io::Write> MessageWriter for WriteLineMessageWriter<T> {
    fn write_message(&mut self, msg: &str) -> Result<(), GError> {
        self.0.write_all(msg.as_bytes())?;
        self.0.write_all(&['\n' as u8])?;
        self.0.flush()?;
        Ok(())
    }
}