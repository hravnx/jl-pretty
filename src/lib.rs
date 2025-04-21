mod ansi;
mod line_processor;
mod session;

use std::path::PathBuf;

pub use line_processor::LineProcessor;
pub use session::SessionStartDetector;

// --------------------------------------------------------------------------

/// Enumerates the kinds of errors that can happen during line processing
pub enum ProcessError {
    /// An error happened during JSON parsing of the individual log lines
    ParseError(usize, sonic_rs::Error),
    /// Could not open input file
    InputNotFound(PathBuf, std::io::Error),
    /// An error happened while reading the input stream
    ReadError(usize, std::io::Error),
    /// An error happened while writing to the output stream
    WriteError(std::io::Error),
}

impl ProcessError {
    /// Creates a new process error from a parse error
    fn from_parse_error(line_no: usize, value: sonic_rs::Error) -> ProcessError {
        ProcessError::ParseError(line_no, value)
    }

    /// Creates a new process error from a read error
    fn from_read_error(line_no: usize, value: std::io::Error) -> ProcessError {
        ProcessError::ReadError(line_no, value)
    }

    /// Creates a new process error from a write error
    fn from_write_error(value: std::io::Error) -> ProcessError {
        ProcessError::WriteError(value)
    }
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(line_no, err) => write!(f, "Parse error on line {line_no}: {err}"),
            Self::InputNotFound(file_path, err) => {
                write!(f, "Could not open input '{}': {err}", file_path.display())
            }
            Self::ReadError(line_no, err) => write!(f, "Read error after line {line_no}: {err}"),
            Self::WriteError(err) => write!(f, "Write error: {err}"),
        }
    }
}

/// Type alias for the result type of the line processor
pub type Result<T> = std::result::Result<T, ProcessError>;
