#[derive(thiserror::Error, Debug)]
pub enum BlueError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("local error: {0}")]
    Local(String),
    #[error("remote error: {0}")]
    Remote(String),
}

impl BlueError {
    pub fn local_err<E>(e: E) -> Self
    where
        E: ToString,
    {
        BlueError::Local(e.to_string())
    }

    pub fn remote_err<E>(e: E) -> Self
    where
        E: ToString,
    {
        BlueError::Remote(e.to_string())
    }
}
