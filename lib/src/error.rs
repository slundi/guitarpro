use std::fmt;

/// Error type for Guitar Pro file parsing
#[derive(Debug)]
pub enum GpError {
    /// Reached end of binary data unexpectedly
    UnexpectedEof { offset: usize, needed: usize },
    /// Invalid enum/flag value encountered during parsing
    InvalidValue { context: &'static str, value: i64 },
    /// String decoding failure
    StringDecode { offset: usize },
    /// ZIP, XML, or format-level errors from GP6/GP7 parsing
    FormatError(String),
    /// IO errors
    Io(std::io::Error),
}

/// Convenience type alias
pub type GpResult<T> = Result<T, GpError>;

impl fmt::Display for GpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpError::UnexpectedEof { offset, needed } => {
                write!(f, "Unexpected end of file at offset {}, needed {} more bytes", offset, needed)
            }
            GpError::InvalidValue { context, value } => {
                write!(f, "Invalid value {} for {}", value, context)
            }
            GpError::StringDecode { offset } => {
                write!(f, "Unable to decode string at offset {}", offset)
            }
            GpError::FormatError(msg) => {
                write!(f, "Format error: {}", msg)
            }
            GpError::Io(err) => {
                write!(f, "IO error: {}", err)
            }
        }
    }
}

impl std::error::Error for GpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GpError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for GpError {
    fn from(err: std::io::Error) -> Self {
        GpError::Io(err)
    }
}

impl From<String> for GpError {
    fn from(msg: String) -> Self {
        GpError::FormatError(msg)
    }
}
