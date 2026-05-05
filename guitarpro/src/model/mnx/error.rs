use thiserror::Error;

#[derive(Error, Debug)]
pub enum MnxIdError {
    #[error("ID exceeds maximum length of 256 characters")]
    TooLong,
    #[error("ID contains invalid characters (must be ASCII and no spaces)")]
    InvalidCharacters,
    #[error("ID cannot be empty")]
    Empty,
}

#[derive(Error, Debug, PartialEq)]
pub enum MnxError {
    #[error("Invalid MIDI number: {0}. Must be between 0 and 127.")]
    InvalidMidiNumber(u8),

    #[error("invalid tie target type: {0}")]
    InvalidTieTarget(String),
}
