use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::rt::{Read, Write};
use pin_project::pin_project;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;
use tokio::net::UnixStream;

/// A passthrough implementation that allows
/// [`tokio`]'s [`UnixStream`] to be used with [`hyper`]
#[pin_project]
pub(super) struct UnixSocketIo {
    #[pin]
    uds: UnixStream,
}

impl Read for UnixSocketIo {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        // This is taken basically from here:
        // https://github.com/hyperium/hyper/blob/90eb95f62a32981cb662b0f750027231d8a2586b/benches/support/tokiort.rs#L104
        // but altered to match my style a little more.
        //
        // It should be correct, all it's doing is reading into the buffer and then advancing the cursor by the read amount.
        //
        // Sadly, there's no way to do this without using `unsafe` :/
        // ~ Zoe
        unsafe {
            let mut tokio_buf = ReadBuf::uninit(buf.as_mut());
            match self.project().uds.poll_read(cx, &mut tokio_buf) {
                ok @ Poll::Ready(Ok(())) => {
                    // NLLs aren't quite good enough to figure out that `tokio_buf`
                    // expires if put in as an argument to `buf.advance` yet
                    let written = tokio_buf.filled().len();
                    buf.advance(written);
                    ok
                }
                anything_else => anything_else,
            }
        }
    }
}

impl Write for UnixSocketIo {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.project().uds.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        self.project().uds.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project().uds.poll_shutdown(cx)
    }
}

impl From<UnixStream> for UnixSocketIo {
    fn from(uds: UnixStream) -> Self {
        Self { uds }
    }
}
