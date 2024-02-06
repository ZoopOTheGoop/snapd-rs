pub mod alias;
pub(crate) mod snap;

use std::marker::PhantomData;

use async_trait::async_trait;
use http::{header::CONTENT_TYPE, request::Builder as RequestBuilder, StatusCode};
use http_body_util::{Collected, Empty};
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use url::Url;

const JSON_CONTENT: &str = "application/json";

use crate::{connection::body::SnapdRequestBody, GetClient, SnapdClientError};

#[doc(inline)]
pub use snap::*;

pub trait Payload<'de>: From<Collected<Bytes>>
where
    Self: 'de,
{
    type Parsed<'a>
    where
        Self: 'a,
        'a: 'de;

    fn parse<'a>(&'a self) -> Self::Parsed<'a>
    where
        'a: 'de;
}

#[async_trait]
pub trait Get: Sized + Sync {
    type Payload<'a>: Payload<'a>;
    type Client: GetClient + Sync;

    async fn get<'a>(&self, client: &Self::Client) -> Result<Self::Payload<'a>, SnapdClientError> {
        client.get(self).await
    }

    fn attach_header(&self, builder: RequestBuilder) -> RequestBuilder {
        builder.header(CONTENT_TYPE, JSON_CONTENT)
    }

    fn url(&self, base_url: Url) -> Url;

    fn to_body(&self) -> SnapdRequestBody {
        SnapdRequestBody::Empty(Empty::default())
    }
}

pub struct JsonPayload<'de, R>
where
    R: Deserialize<'de>,
{
    pub data: Bytes,
    pd: PhantomData<&'de R>,
}

impl<'de, R> JsonPayload<'de, R>
where
    R: Deserialize<'de>,
{
    pub fn parse(&'de self) -> Result<R, serde_json::Error> {
        println!("{}", String::from_utf8(self.data.to_vec()).unwrap());
        let parsed: SnapdResponse<R> = serde_json::from_slice(&self.data)?;
        Ok(parsed.result)
    }
}

impl<'de, R> Payload<'de> for JsonPayload<'de, R>
where
    R: Deserialize<'de>,
{
    type Parsed<'a> = R where Self: 'a, 'a: 'de;

    fn parse<'a>(&'a self) -> Self::Parsed<'a>
    where
        'a: 'de,
        Self: 'a,
    {
        self.parse().expect(
            "error in parsing response json, this is an \
        internal snapd-rs bug, please file an issue",
        )
    }
}

impl<'de, R> From<Collected<Bytes>> for JsonPayload<'de, R>
where
    R: Deserialize<'de>,
{
    fn from(data: Collected<Bytes>) -> Self {
        Self {
            data: data.to_bytes(),
            pd: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
enum SnapdType {
    Sync,
    Async,
}

#[derive(Clone, Hash, Eq, PartialEq, Deserialize)]
struct SnapdResponse<T> {
    #[serde(rename = "type")]
    typ: SnapdType,
    #[serde(rename = "status-code")]
    status_code: StatusCodeProxy,
    // Deliberately ignoring status because (at least for now), we can infer from `status_code`
    result: T,
}

#[derive(Clone, Hash, Eq, PartialEq, Deserialize)]
#[serde(from = "u16")]
struct StatusCodeProxy(StatusCode);

impl From<u16> for StatusCodeProxy {
    fn from(val: u16) -> StatusCodeProxy {
        // We're just going to assume `snapd` sends actually valid status codes
        StatusCodeProxy(StatusCode::try_from(val).unwrap())
    }
}

macro_rules! snap_str_newtype {
    ($(#[$attr:meta])*$typename:ident) => {
        $(#[$attr])*
        #[derive(
            Clone, serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Default,
        )]
        #[serde(transparent)]
        pub struct $typename<'a>(#[serde(borrow)] std::borrow::Cow<'a, str>);

        impl<'a> AsRef<str> for $typename<'a> {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl<'a> std::borrow::Borrow<str> for $typename<'a> {
            fn borrow(&self) -> &str {
                self.0.borrow()
            }
        }

        impl<'a> From<&'a str> for $typename<'a> {
            fn from(val: &'a str) -> Self {
                Self(val.into())
            }
        }

        impl<'a> From<String> for $typename<'a> {
            fn from(val: String) -> Self {
                Self(val.into())
            }
        }

        impl<'a> std::fmt::Display for $typename<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_ref())
            }
        }

        impl<'a> $crate::api::snap::ToOwnedInner for $typename<'a> {
            type Other<'b> = $typename<'b>;

            fn to_owned_inner<'b>(self) -> Self::Other<'b> {
                $typename(self.0.into_owned().into())
            }
        }


    };

    ($($(#[$attr:meta])*$typename:ident),+) => {
        $(snap_str_newtype!{$(#[$attr])*$typename})*
    };
}

// This has to be stated *after* the definition for macros
use snap_str_newtype;
