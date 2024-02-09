use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{SnapdClient, SnapdClientError};

use super::{snap_str_newtype, Get, JsonPayload, SnapId, SnapName, ToOwnedInner};

#[derive(Clone, Debug, Error)]
pub enum FindError {
    #[error("didn't find a snap with the given id")]
    NoSnapsFound,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FindSnapByName<'a> {
    pub name: SnapName<'a>,
}

impl<'a> FindSnapByName<'a> {
    pub async fn get_categories<'b, 'c>(
        name: SnapName<'b>,
        client: &SnapdClient,
    ) -> Result<Vec<StoreCategory<'c>>, SnapdClientError> {
        let payload = FindSnapByName { name }.get(client).await?;
        let mut snaps = payload.parse().unwrap();
        if snaps.info.is_empty() {
            return Err(FindError::NoSnapsFound)?;
        }
        debug_assert_eq!(
            snaps.info.len(),
            1,
            "filtering by name somehow returned more than one snap?"
        );

        let categories: Vec<_> = snaps
            .info
            .pop()
            .unwrap()
            .categories
            .into_iter()
            .map(|v| v.to_owned_inner())
            .collect();

        Ok(categories)
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FindSnapById<'a> {
    pub id: SnapId<'a>,
}

impl<'a> FindSnapById<'a> {
    pub async fn get_categories<'b, 'c>(
        id: SnapId<'b>,
        client: &SnapdClient,
    ) -> Result<Vec<StoreCategory<'c>>, SnapdClientError> {
        let payload = FindSnapById { id }.get(client).await?;
        let mut snaps = payload.parse().expect("snapd returned invalid json?");
        if snaps.info.is_empty() {
            return Err(FindError::NoSnapsFound)?;
        }
        debug_assert_eq!(
            snaps.info.len(),
            1,
            "filtering by ID somehow returned more than one snap?"
        );

        let categories: Vec<_> = snaps
            .info
            .pop()
            .unwrap()
            .categories
            .into_iter()
            .map(|v| v.to_owned_inner())
            .collect();

        Ok(categories)
    }
}

impl<'a> Get for FindSnapById<'a> {
    type Payload<'de> = JsonPayload<'de, FindResult<'de>>;

    type Client = SnapdClient;

    fn url(&self, base_url: Url) -> Url {
        base_url
            .join(&format!("/v2/find?common-id={}", self.id))
            .expect("error formatting snap find URL, internal error")
    }
}

#[derive(Serialize, Deserialize, Hash, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct FindResult<'a> {
    #[serde(borrow)]
    pub info: Vec<SnapInfo<'a>>,
}

#[derive(Serialize, Deserialize, Hash, Clone, PartialEq, Eq)]
pub struct SnapInfo<'a> {
    #[serde(borrow)]
    pub id: SnapId<'a>,
    #[serde(borrow)]
    pub title: SnapTitle<'a>,
    #[serde(borrow)]
    pub summary: Summary<'a>,
    #[serde(borrow)]
    pub description: Description<'a>,
    #[serde(borrow)]
    pub name: SnapName<'a>,
    #[serde(borrow)]
    pub developer: Developer<'a>,
    #[serde(borrow)]
    pub categories: Vec<StoreCategory<'a>>,
}

#[derive(Serialize, Deserialize, Hash, Clone, PartialEq, Eq)]
pub struct StoreCategory<'a> {
    #[serde(borrow)]
    pub name: CategoryName<'a>,
    pub featured: bool,
}

impl<'a> ToOwnedInner for StoreCategory<'a> {
    type Other = StoreCategory<'static>;

    fn to_owned_inner<'b>(self) -> Self::Other {
        StoreCategory {
            name: self.name.to_owned_inner(),
            featured: self.featured,
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

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::{FindSnapById, FindSnapByName};
    use crate::SnapdClient;

    #[tokio::test]
    async fn categories_from_name() {
        let categories =
            FindSnapByName::get_categories("colorgrab".into(), &SnapdClient::default())
                .await
                .unwrap();

        let set: HashSet<_> = categories
            .iter()
            .map(|category| category.name.0.as_ref())
            .collect();

        let expected: HashSet<_> =
            HashSet::from_iter(vec!["art-and-design", "utilities"].into_iter());

        assert_eq!(set, expected)
    }

    #[tokio::test]
    async fn categories_from_id() {
        let categories = FindSnapById::get_categories(
            "3Iwi803Tk3KQwyD6jFiAJdlq8MLgBIoD".into(),
            &SnapdClient::default(),
        )
        .await
        .unwrap();

        let set: HashSet<_> = categories
            .iter()
            .map(|category| category.name.0.as_ref())
            .collect();

        let expected: HashSet<_> =
            HashSet::from_iter(vec!["art-and-design", "utilities"].into_iter());

        assert_eq!(set, expected)
    }
}
