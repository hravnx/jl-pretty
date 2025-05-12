use std::fmt;
use std::fmt::Write;
use std::io;
use thiserror::Error;

use crate::params::DataSize as DS;
use crate::params::Params;

// --------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("line generate error")]
    GenerateError(#[from] fmt::Error),

    #[error("line write error")]
    WriteError(#[from] io::Error),
}

// --------------------------------------------------------------------------

pub struct GeneratorStats {
    pub lines_generated: u64,
    pub bytes_generated: u64,
}

pub type Result<T> = std::result::Result<T, GeneratorError>;

pub trait LineGenerator {
    fn generate_line(&self, w: &mut String) -> Result<()>;
}

pub struct WinstonLineGenerator;

impl LineGenerator for WinstonLineGenerator {
    fn generate_line(&self, w: &mut String) -> Result<()> {
        todo!()
    }
}

pub struct Generator {}

impl Generator {
    pub fn generate<W>(&self, w: &mut W, params: &Params) -> Result<GeneratorStats>
    where
        W: std::io::Write,
    {
        let mut is_done = false;
        let mut bytes_generated = 0u64;
        let mut lines_generated = 0u64;
        let mut line_buffer = String::with_capacity(1024);

        while !is_done {
            line_buffer.clear();

            //self.generate_line(&mut line_buffer, params.template())?;

            line_buffer.push_str("\n");
            w.write_all(line_buffer.as_bytes())?;

            lines_generated += 1;
            bytes_generated += line_buffer.len() as u64 + 1u64;

            // now we can check if we're done ...
            is_done = match params.data_size() {
                DS::Lines(line_count) => lines_generated >= *line_count,
                DS::AtLeast(byte_count) => bytes_generated >= *byte_count,
            }
        }

        Ok(GeneratorStats {
            lines_generated,
            bytes_generated,
        })
    }

    /// Generate a single line of log data, according to the given template
    fn generate_line<W>(&self, line_buffer: &mut W, template: &str) -> Result<String>
    where
        W: Write,
    {
        let next_line = "";
        line_buffer.write_str(next_line)?;
        todo!()
    }
}
