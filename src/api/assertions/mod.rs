use std::{convert::Infallible, io::BufRead, marker::PhantomData};

use http_body_util::Collected;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Payload, SnapId, SnapName, ToOwnedInner};

#[derive(Clone, Debug, Error)]
pub enum SnapDeclarationError {
    #[error("didn't find a snap with the given id")]
    NoSnapsFound,
}

#[derive(Clone, Default, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct SnapDeclaration<'a> {
    #[serde(borrow)]
    pub(crate) snap_id: SnapId<'a>,
    #[serde(borrow)]
    pub(crate) snap_name: SnapName<'a>,
}

pub struct DeclarationAssertionPayload<'de> {
    pub data: Bytes,
    pd: PhantomData<&'de SnapDeclaration<'de>>,
}

impl<'de> DeclarationAssertionPayload<'de> {
    pub(crate) fn parse(&'de self) -> Result<SnapDeclaration<'de>, Infallible> {
        let mut declaration = SnapDeclaration::default();
        // Super annoying, need to fix this, but the assertion response is a huge mess to begin with
        for line in self.data.lines().map(|v| v.unwrap()) {
            if line.starts_with("snap-name") {
                let name: SnapName = line.split_once(':').unwrap().1.trim().into();
                declaration.snap_name = name.to_owned_inner();
            }

            if line.starts_with("snap-id") {
                let id: SnapId = line.split_once(':').unwrap().1.trim().into();
                declaration.snap_id = id.to_owned_inner();
            }
        }

        Ok(declaration)
    }
}

impl<'de> Payload<'de> for DeclarationAssertionPayload<'de> {
    type Parsed<'a> = SnapDeclaration<'a> where Self: 'a, 'a: 'de;

    fn parse<'a>(&'a self) -> Self::Parsed<'a>
    where
        'a: 'de,
        Self: 'a,
    {
        self.parse().expect(
            "error in parsing assertion response, this is an \
        internal snapd-rs bug, please file an issue",
        )
    }
}

impl<'de> From<Collected<Bytes>> for DeclarationAssertionPayload<'de> {
    fn from(data: Collected<Bytes>) -> Self {
        Self {
            data: data.to_bytes(),
            pd: PhantomData,
        }
    }
}
