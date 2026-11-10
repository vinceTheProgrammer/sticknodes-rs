use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LibraryError {
    #[error("{0}")]
    AnyString(String),

    #[error(
        "Stick Nodes version {0} is not supported. Please check if an update to this library is available."
    )]
    UnsupportedVersion(i32),

    #[error(
        "Although Stick Nodes version {0} is supported, Stick Nodes build {1} is not supported. Please check if an update to this library is available."
    )]
    UnsupportedBuild(i32, i32),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Decompression error: {0:?}")]
    Decompress(miniz_oxide::inflate::DecompressError),

    #[error("Stickfigure file error: {0}")]
    StickfigureError(#[from] StickfigureError),
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StickfigureError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    GenericError(String),

    #[error("Invalid stickfigure header: {0}")]
    InvalidHeader(String),

    #[error("Error reading node: {0}")]
    NodeError(String),

    #[error("Cannot add {0} node(s). Current node count is {1}. Node limit is {2}.")]
    NodeLimitError(usize, usize, usize),

    #[error("Draw order index {0} does not exist. {1}")]
    InvalidDrawIndex(i32, String),

    #[error("Node at draw order index {0} is already an anchor node. {1}")]
    NodeIsAlreadyAnchor(i32, String),

    #[error("The following draw order indices do not exist: {0} {1}")]
    InvalidDrawIndices(String, String),

    #[error("Draw order index {0} is already occupied. {1}")]
    OccupiedDrawIndex(i32, String),
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ColorError {
    #[error(
        "{0} is not a valid hex string. The value of the following part of the hex string could not be parsed: {1}"
    )]
    InvalidHexStringValue(String, String),

    #[error("{0} is not a valid hex string. It has an invalid length: {1}.")]
    InvalidHexStringLength(String, usize),

    #[error("{0} is not a valid hex string. It is missing the starting '#' prefix.")]
    InvalidHexStringPrefix(String),

    #[error("{0} is not a valid hex string.")]
    InvalidHexString(String),

    #[error("Hex string is empty.")]
    EmptyHexString,

    #[error("Trimmed hex string is empty. Was your hex string just '#'s?")]
    EmptyTrimmedHexString,
}

impl From<miniz_oxide::inflate::DecompressError> for LibraryError {
    fn from(error: miniz_oxide::inflate::DecompressError) -> Self {
        Self::Decompress(error)
    }
}
