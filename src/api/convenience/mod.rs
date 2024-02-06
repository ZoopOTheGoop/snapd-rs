use url::Url;

use crate::{GetClient, SnapdClient};

use super::assertions::DeclarationAssertionPayload;
use super::{Get, SnapId, SnapName, ToOwnedInner};

#[derive(Clone, Default, Hash, Eq, PartialEq, Debug)]
pub struct SnapIdFromName<'a> {
    pub name: SnapName<'a>,
}

impl<'a> SnapIdFromName<'a> {
    pub async fn get_id<'b, 'c>(name: SnapName<'b>, client: &SnapdClient) -> SnapId<'c> {
        client
            .get(&SnapIdFromName { name })
            .await
            .unwrap()
            .parse()
            .unwrap()
            .snap_id
            .to_owned_inner()
    }
}

impl<'a> Get for SnapIdFromName<'a> {
    type Payload<'de> = DeclarationAssertionPayload<'de>;

    type Client = SnapdClient;

    fn url(&self, base_url: Url) -> Url {
        base_url
            .join(&format!(
                "/v2/assertions/snap-declaration?snap-name={}",
                self.name
            ))
            .unwrap()
    }
}

#[derive(Clone, Default, Hash, Eq, PartialEq, Debug)]
pub struct SnapNameFromId<'a> {
    pub name: SnapId<'a>,
}

impl<'a> SnapNameFromId<'a> {
    pub async fn get_name<'b, 'c>(id: SnapId<'b>, client: &SnapdClient) -> SnapName<'c> {
        client
            .get(&SnapNameFromId { name: id })
            .await
            .unwrap()
            .parse()
            .unwrap()
            .snap_name
            .to_owned_inner()
    }
}

impl<'a> Get for SnapNameFromId<'a> {
    type Payload<'de> = DeclarationAssertionPayload<'de>;

    type Client = SnapdClient;

    fn url(&self, base_url: Url) -> Url {
        base_url
            .join(&format!(
                "/v2/assertions/snap-declaration?snap-id={}",
                self.name
            ))
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn id_from_name() {
        let client = SnapdClient::default();

        let id = SnapIdFromName::get_id("steam".into(), &client).await;
        assert_eq!("NeoQngJVBf2wKC48bxnF2xqmfEFGdVnx", id.as_ref())
    }

    #[tokio::test]
    async fn name_from_id() {
        let client = SnapdClient::default();

        let id = SnapNameFromId::get_name("NeoQngJVBf2wKC48bxnF2xqmfEFGdVnx".into(), &client).await;
        assert_eq!("steam", id.as_ref())
    }
}
