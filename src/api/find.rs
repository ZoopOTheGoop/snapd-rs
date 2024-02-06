use serde::{Deserialize, Serialize};
use url::Url;

use crate::SnapdClient;

use super::{snap_str_newtype, Get, JsonPayload, SnapId, SnapName, ToOwnedInner};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FindSnapByName<'a> {
    name: SnapName<'a>,
}

impl<'a> FindSnapByName<'a> {
    pub async fn get_categories<'c>(&self, client: &SnapdClient) -> Vec<StoreCategory<'c>> {
        let payload = self.get(client).await.unwrap();
        let mut snaps = payload.parse().unwrap();
        debug_assert_eq!(snaps.info.len(), 1);

        let categories: Vec<_> = snaps
            .info
            .pop()
            .unwrap()
            .categories
            .into_iter()
            .map(|v| v.to_owned_inner())
            .collect();

        categories
    }
}

impl<'a> Get for FindSnapByName<'a> {
    type Payload<'de> = JsonPayload<'de, FindResult<'de>>;

    type Client = SnapdClient;

    fn url(&self, base_url: Url) -> Url {
        base_url
            .join(&format!("/v2/find?name={}", self.name))
            .expect("error formatting snap find URL, internal error")
    }
}

#[derive(Serialize, Deserialize, Hash, Clone, PartialEq, Eq)]
pub struct FindResult<'a> {
    #[serde(flatten, borrow)]
    info: Vec<SnapInfo<'a>>,
}

#[derive(Serialize, Deserialize, Hash, Clone, PartialEq, Eq)]
pub struct SnapInfo<'a> {
    #[serde(borrow)]
    id: SnapId<'a>,
    #[serde(borrow)]
    title: SnapTitle<'a>,
    #[serde(borrow)]
    summary: Summary<'a>,
    #[serde(borrow)]
    description: Description<'a>,
    #[serde(borrow)]
    name: SnapName<'a>,
    #[serde(borrow)]
    developer: Developer<'a>,
    #[serde(borrow)]
    categories: Vec<StoreCategory<'a>>,
}

#[derive(Serialize, Deserialize, Hash, Clone, PartialEq, Eq)]
pub struct StoreCategory<'a> {
    #[serde(borrow)]
    name: CategoryName<'a>,
    features: bool,
}

impl<'a> ToOwnedInner for StoreCategory<'a> {
    type Other<'b> = StoreCategory<'b>;

    fn to_owned_inner<'b>(self) -> Self::Other<'b> {
        StoreCategory {
            name: self.name.to_owned_inner(),
            features: self.features,
        }
    }
}

snap_str_newtype! {
    SnapTitle,
    Summary,
    Description,
    Developer,
    CategoryName
}
