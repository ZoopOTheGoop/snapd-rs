use url::Url;

use crate::{GetClient, SnapdClient};

use super::assertions::DeclarationAssertionPayload;
use super::{Get, SnapId, SnapName, ToOwnedInner};

#[derive(Clone, Default, Hash, Eq, PartialEq, Debug)]
pub struct SnapNameFromId<'a> {
    pub name: SnapId<'a>,
}

impl<'a> SnapNameFromId<'a> {
    pub async fn get_name(id: SnapId<'_>, client: &SnapdClient) -> SnapName<'static> {
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
                // TODO: understand implications of `series=16` but it seems to work for
                // a wide variety of snaps I've tested ATM
                "/v2/assertions/snap-declaration?series=16&remote=true&snap-id={}",
                self.name
            ))
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn name_from_id() {
        let client = SnapdClient::default();

        let id = SnapNameFromId::get_name("NeoQngJVBf2wKC48bxnF2xqmfEFGdVnx".into(), &client).await;
        assert_eq!("steam", id.as_ref())
    }
}
