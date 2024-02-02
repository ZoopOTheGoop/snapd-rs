use http_body_util::Empty;
use hyper::body::{Body, Bytes};
use pin_project::pin_project;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BodyPollError {}

#[derive(Debug, Clone)]
#[pin_project(project = SRBProject)]
pub(crate) enum SnapdRequestBody {
    Empty(#[pin] Empty<Bytes>),
}

impl Default for SnapdRequestBody {
    fn default() -> Self {
        Self::Empty(Empty::new())
    }
}

impl Body for SnapdRequestBody {
    type Data = Bytes;

    type Error = BodyPollError;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        match self.project() {
            SRBProject::Empty(val) => val
                .poll_frame(cx)
                .map_err(|_| unreachable!("The error type is literally 'Infallible'")),
        }
    }
}
