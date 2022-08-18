mod error;
pub use error::*;

pub type BlueResult<T> = std::result::Result<T, BlueError>;
