#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    InvalidType,
    InvalidData,
}

pub type CodecResult<T> = Result<T, CodecError>;
