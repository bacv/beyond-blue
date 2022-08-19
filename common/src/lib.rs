mod error;
mod identity;

pub use error::*;
pub use identity::*;

pub type BlueResult<T> = std::result::Result<T, BlueError>;
