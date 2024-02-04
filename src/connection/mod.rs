use std::mem;

use deadpool::managed::{Metrics, RecycleResult};
use deadpool::{async_trait, managed::Manager};
use hyper::client::conn::http1::{self as conn, SendRequest};
use tokio::net::UnixStream;
use tokio::task::JoinHandle;

mod body;
mod error;
mod io;

use body::SnapdRequestBody;
use io::UnixSocketIo;

#[doc(inline)]
pub use error::{ConnectionReuseError, SnapdConnectionError};

const SNAPD_SOCKET_PATH: &str = "/run/snapd.socket";

pub(crate) enum SnapdConnection {
    Active {
        request_sender: SendRequest<SnapdRequestBody>,
        connection_join_handle: JoinHandle<Result<(), hyper::Error>>,
    },
    Closed,
}

impl SnapdConnection {
    /// Creates a new live connection to the `snapd` socket. This does not
    /// specify a URI or API endpoint yet.
    async fn new() -> Result<Self, SnapdConnectionError> {
        let stream = UnixSocketIo::from(UnixStream::connect(SNAPD_SOCKET_PATH).await.unwrap());

        let (request_sender, connection) = conn::handshake::<_, SnapdRequestBody>(stream).await?;

        let connection_join_handle = tokio::spawn(connection);

        Ok(Self::Active {
            request_sender,
            connection_join_handle,
        })
    }

    /// Checks if the sender got closed (probably by the remote host).
    fn is_closed(&self) -> bool {
        match self {
            Self::Active { request_sender, .. } => request_sender.is_closed(),
            _ => false,
        }
    }

    /// Checks if the coroutine is finished, which should only happen on error
    /// given there's no way to drop `sender`.
    fn is_finished(&self) -> bool {
        match self {
            Self::Active {
                connection_join_handle,
                ..
            } => connection_join_handle.is_finished(),
            _ => false,
        }
    }

    /// Determines whether the connection has ended for some reason
    /// (e.g. an error or `snapd` closed it).
    fn connection_ended(&self) -> bool {
        self.is_closed() || self.is_finished()
    }

    /// Closes the sender and joins the connection coroutine, checking for errors.
    async fn close(self) -> Result<(), ConnectionReuseError> {
        let (sender, join_handle) = match self {
            Self::Closed => return Ok(()),
            Self::Active {
                connection_join_handle,
                request_sender,
            } => (request_sender, connection_join_handle),
        };

        // There's basically two cases here: either we have no error,
        // in which case `join_handle.await` will hang literally forever unless the sender
        // is dropped to make the coroutine hang up cleanly, or there's already an error
        // and dropping the sender has absolutely no effect whatsoever and would be done at
        // the end of the function anyway.
        //
        // Technically this would've been done for us if we had just not returned it from
        // the `match` block above, but this drop
        // is important enough it was worth highlighting by doing explicitly.
        drop(sender);

        Ok(join_handle.await??)
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
pub(crate) struct SnapdConnectionManager {}

#[async_trait]
impl Manager for SnapdConnectionManager {
    type Type = SnapdConnection;
    type Error = SnapdConnectionError;

    async fn create(&self) -> Result<SnapdConnection, SnapdConnectionError> {
        SnapdConnection::new().await
    }

    async fn recycle(
        &self,
        conn: &mut SnapdConnection,
        _: &Metrics,
    ) -> RecycleResult<SnapdConnectionError> {
        match conn {
            SnapdConnection::Closed => Err(ConnectionReuseError::NaturallyClosed)?,
            snapd_conn @ SnapdConnection::Active { .. } => {
                if snapd_conn.connection_ended() {
                    let mut new = SnapdConnection::Closed;

                    mem::swap(snapd_conn, &mut new);
                    let extracted = new;
                    extracted.close().await?;

                    Err(ConnectionReuseError::NaturallyClosed.into())
                } else {
                    Ok(())
                }
            }
        }
    }
}
