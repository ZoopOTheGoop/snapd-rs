//! These represent core types used in many snap contexts, stuff like a Snap's name or the apps it contains.
//!
//! All these types implement [`Serialize`](serde::Serialize) and [`Deserialize`] so they can be written as `json`
//! and decoded from such over the wire. They also all implement zero-allocation deserialization. As long as the json
//! string outlives the type, it will simply point to the json.
//!
//!
//! For instance, this should fail, since the `json` drops before we use our [`SnapName`]:
//! ```compile_fail
//! # use snapd::api::SnapName;
//! let name = {
//!     let json: String = "\"foo\"".to_owned();
//!
//!     serde_json::from_str::<SnapName>(&json).unwrap()
//! };
//!
//! assert_eq!("foo", name.as_ref());
//! ```
//!
//! However, you can always make it outlive the `json` by taking ownership of the inner
//! values with the [`ToOwnedInner`] trait:
//! ```
//! # use snapd::api::SnapName;
//! use snapd::api::ToOwnedInner;
//! let name = {
//!     let json: String = "\"foo\"".to_owned();
//!
//!     serde_json::from_str::<SnapName>(&json).unwrap().to_owned_inner()
//! };
//!
//! assert_eq!("foo", name.as_ref());
//! ```

use std::{borrow::Cow, fmt::Display};

use serde::{Deserialize, Serialize};

use super::snap_str_newtype;

pub trait ToOwnedInner {
    type Other;

    fn to_owned_inner(self) -> Self::Other;
}

impl<'a, T> ToOwnedInner for Cow<'a, T>
where
    T: ?Sized + 'a + ToOwned + 'static,
    <T as ToOwned>::Owned: Clone + 'static,
{
    type Other = Cow<'static, T>;

    fn to_owned_inner(self) -> Self::Other {
        Cow::Owned(self.into_owned())
    }
}

snap_str_newtype! {
    /// A Snap's Name, for instance `steam` represents the name of the `steam` snap.
    ///
    /// This is guaranteed to be unique, but in rare circumstances may change if the creator of
    /// the snap changes the snap name.
    ///
    /// Use [`SnapId`] if you want a value guaranteed to be unique (though be aware fewer API endpoints
    /// allow the ID as input at the moment).
    SnapName,

    /// The representation of a Snap's apps, or *just* the command portion of a Snap command. Snaps are namespaced
    /// but can have multiple commands. In the `lxd` snap, there is a subcommand for `lxc`,
    /// and this is namespaced as `lxd.lxc`. In this example, the value of this is `lxc`.
    App,

    /// A Snap's unique ID. This will always be the same no matter what happens to the Snap, and will never
    /// collide with another Snap. However, few API endpoints take the ID at this time, you may want to look
    /// up the corresponding [`SnapName`] if you don't have it.
    SnapId
}

/// A Snap Command. Every Snap has one or mode "apps" that are namespaced under the Snap itself. For instance,
/// the `lxd` Snap also contains `lxc` as a subprogram. These are then namespaced as `lxd.lxc`.  
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Eq, Default)]
#[serde(from = "&str")]
pub struct SnapCommand<'a, 'b> {
    #[serde(borrow)]
    pub name: SnapName<'a>,
    #[serde(borrow)]
    pub command: Option<App<'b>>,
}

impl<'a, 'b> SnapCommand<'a, 'b> {
    pub fn name_only(name: SnapName<'a>) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn from_parts(name: SnapName<'a>, command: App<'b>) -> Self {
        Self {
            name,
            command: Some(command),
        }
    }

    pub fn from_convertible<N: Into<SnapName<'a>>, C: Into<App<'b>>>(name: N, command: C) -> Self {
        Self::from_parts(name.into(), command.into())
    }
}

impl<'a> SnapCommand<'a, 'a> {
    pub fn from_raw(raw_command: &'a str) -> Self {
        raw_command
            .split_once('.')
            .map(|(name, command)| Self {
                name: name.into(),
                command: Some(command.into()),
            })
            .unwrap_or(Self {
                name: raw_command.into(),
                command: None,
            })
    }

    pub fn from_raw_owned(raw_command: String) -> Self {
        // Note: it is *very* important this be `SnapCommand::from_raw` instead of `Self::from_raw`;
        // it took me forever to debug this, but `Self` implies the same lifetimes, which leads to an
        // infinite loop where `Self::from_raw` followed by converting the values into strings forces
        // the compiler to assume that the borrow of `raw_command` from `from_raw`
        // somehow outlives the lifetime of `String`.
        //
        // Also, there's really no good way to do this without two clones, at least not with severely
        // overcomplicating this struct. We could *technically* have an underlying `raw` that's `Pin`ned
        // and `Box`ed, as well as an `Option`` in case we use `from_parts` to construct this. And then like
        // put `raw_command` on the heap in a pinned location and thus avoid two allocations. Or worse options
        // with `unsafe`.
        //
        // But like... why bother?
        SnapCommand::from_raw(&raw_command).to_owned_inner()
    }
}

impl<'a> From<&'a str> for SnapCommand<'a, 'a> {
    fn from(val: &'a str) -> Self {
        Self::from_raw(val)
    }
}

impl<'a, 'b> ToOwnedInner for SnapCommand<'a, 'b> {
    type Other = SnapCommand<'static, 'static>;

    fn to_owned_inner(self) -> Self::Other {
        SnapCommand {
            name: self.name.to_owned_inner(),
            command: self.command.map(|v| v.to_owned_inner()),
        }
    }
}

impl<'a, 'b> Display for SnapCommand<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref command) = self.command {
            write!(f, "{}.{}", self.name.as_ref(), command.as_ref())
        } else {
            write!(f, "{}", self.name.as_ref())
        }
    }
}

impl<'a, 'b> Serialize for SnapCommand<'a, 'b> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod test {
    use super::SnapCommand;
    use serde_json;

    #[test]
    fn serialize_command() {
        assert_eq!(
            serde_json::to_string(&SnapCommand::from_convertible("lxd", "lxc"))
                .expect("could not serialize snap command"),
            "\"lxd.lxc\""
        )
    }

    #[test]
    fn deserialize_command() {
        assert_eq!(
            serde_json::from_str::<SnapCommand>("\"lxd.lxc\"")
                .expect("could not deserialize snap command"),
            SnapCommand::from_convertible("lxd", "lxc")
        )
    }
}
