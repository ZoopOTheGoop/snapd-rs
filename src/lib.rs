use async_trait::async_trait;
use connection::{body::SnapdRequestBody, SnapdConnectionManager};
use deadpool::managed::Pool;
use http::{header::HOST, request::Builder as RequestBuilder};
use hyper::Request;
use thiserror::Error;
use url::Url;

pub mod api;
mod connection;

use api::Get;

#[derive(Debug, Error)]
#[error("A snapd client error happened")]
pub struct SnapdClientError;

#[async_trait]
pub trait GetClient {
    fn attach_header(&self, builder: RequestBuilder) -> RequestBuilder {
        builder.header(HOST, "localhost")
    }

    async fn get<'a, G: Get + Sync>(&self, request: &G)
        -> Result<G::Payload<'a>, SnapdClientError>;

    fn build_request<G: Get>(&self, request: &G) -> Request<SnapdRequestBody> {
        let builder = Request::get(
            request
                .url(Url::parse("http://localhost/").unwrap())
                .as_str(),
        );
        println!("{}", builder.uri_ref().unwrap());
        let builder = request.attach_header(self.attach_header(builder));

        builder.body(request.to_body()).expect(
            "can't make internal request into body? \
        something is wrong with the `snapd-rs` library, please file an issue",
        )
    }
}

#[derive(Debug, Clone)]
pub struct SnapdClient {
    pool: Pool<SnapdConnectionManager>,
}

impl SnapdClient {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl GetClient for SnapdClient {
    async fn get<'a, G: Get + Sync>(
        &self,
        request: &G,
    ) -> Result<G::Payload<'a>, SnapdClientError> {
        let response_json = self
            .pool
            .get()
            .await
            .unwrap()
            .request_response(self.build_request(request))
            .await
            .unwrap()
            .into();

        Ok(response_json)
    }
}

impl Default for SnapdClient {
    fn default() -> Self {
        Self {
            pool: Pool::builder(SnapdConnectionManager)
                .max_size(16)
                .build()
                .expect(
                    "error making connection pool, this is a snapd-rs bug, please file an issue",
                ),
        }
    }
}

#[cfg(test)]
mod test {
    use self::api::alias::GetAliases;

    use super::*;

    // Test both routes and verify they give the same result
    #[tokio::test]
    async fn basic_get() {
        let client = SnapdClient::new();

        let payload = GetAliases.get(&client).await.unwrap();
        let aliases = payload.parse().unwrap();

        let payload = client.get(&GetAliases).await.unwrap();
        let aliases_2 = payload.parse().unwrap();

        assert_eq!(aliases, aliases_2);
    }
}
