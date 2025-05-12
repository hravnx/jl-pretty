use crate::{
    Cli,
    generate::{LineGenerator, WinstonLineGenerator},
};

// --------------------------------------------------------------------------

#[derive(Debug)]
pub enum DataSize {
    /// Generate this number of lines
    Lines(u64),
    /// Generate at least this number of bytes
    AtLeast(u64),
}

impl Default for DataSize {
    fn default() -> Self {
        Self::Lines(100)
    }
}

// --------------------------------------------------------------------------

pub enum Templates {
    Winston,
}

// --------------------------------------------------------------------------
// region:    --- Params

pub struct Params {
    data_size: DataSize,
    line_generator: Box<dyn LineGenerator>,
}

impl Params {
    pub fn new(data_size: DataSize, line_generator: impl LineGenerator) -> Self {
        Self {
            data_size,
            line_generator: Box::new(WinstonLineGenerator),
        }
    }

    pub fn data_size(&self) -> &DataSize {
        &self.data_size
    }

    pub fn line_generator(&self) -> &dyn LineGenerator {
        self.line_generator.as_ref()
    }
}

impl From<Cli> for Params {
    fn from(value: Cli) -> Self {
        let data_size = if let Some(byte_size) = value.byte_size {
            DataSize::AtLeast(byte_size)
        } else if let Some(line_count) = value.line_count {
            DataSize::Lines(line_count)
        } else {
            DataSize::default()
        };

        let template = value.template.unwrap_or_else(|| "winston".to_string());

        Params::new(data_size, WinstonLineGenerator {})
    }
}

// endregion: --- Params
