use deadpool::managed::RecycleError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum SnapdConnectionError {
    #[error("there was a problem during the initial connection handshake: {0}")]
    HandshakeError(#[from] hyper::Error),
    #[error("there was an error reusing a previous connection: {0}")]
    ConnectionReuseError(#[from] ConnectionReuseError),
}

#[derive(Debug, Error)]
pub enum ConnectionReuseError {
    #[error("the connection coroutine to the snapd socket panicked: {0}")]
    ConnectionPanicked(#[from] JoinError),
    #[error("the connection coroutine to the snapd socket encountered an error: {0}")]
    RuntimeError(#[from] hyper::Error),
    #[error("the connection was closed, but not removed from the pool")]
    NaturallyClosed,
}

impl From<ConnectionReuseError> for RecycleError<SnapdConnectionError> {
    fn from(err: ConnectionReuseError) -> Self {
        RecycleError::Backend(SnapdConnectionError::from(err))
    }
}
