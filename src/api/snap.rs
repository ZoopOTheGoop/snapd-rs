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

use core::fmt;
use std::{borrow::Cow, fmt::Display};

use paste::paste;
use serde::de;
use serde::{de::Visitor, Deserialize, Serialize};
use thiserror::Error;

pub trait ToOwnedInner {
    type Other<'b>;

    fn to_owned_inner<'b>(self) -> Self::Other<'b>;
}

macro_rules! snap_str_newtype {
    ($(#[$attr:meta])*$typename:ident) => {
        $(#[$attr])*
        #[derive(
            Clone, serde::Serialize, Debug, Hash, PartialEq, Eq, Default,
        )]
        pub struct $typename<'a>(std::borrow::Cow<'a, str>);

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

        impl<'a> ToOwnedInner for $typename<'a> {
            type Other<'b> = $typename<'b>;

            fn to_owned_inner<'b>(self) -> Self::Other<'b> {
                $typename(self.0.into_owned().into())
            }
        }

        paste! {
            struct [<$typename Visitor>];


            impl<'de> Visitor<'de> for [<$typename Visitor>] {
                type Value = $typename<'de>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a plain string")
                }

                fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
                    Ok(v.into())
                }
            }

            impl<'de, 'a> Deserialize<'de> for $typename<'a>
            where
                'de: 'a,
            {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_str([<$typename Visitor>])
                }
            }
        }


    };

    ($($(#[$attr:meta])*$typename:ident),+) => {
        $(snap_str_newtype!{$(#[$attr])*$typename})*
    };
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
#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct SnapCommand<'a, 'b> {
    pub name: SnapName<'a>,
    pub command: App<'b>,
}

impl<'a, 'b> SnapCommand<'a, 'b> {
    pub fn from_parts(name: SnapName<'a>, command: App<'b>) -> Self {
        Self { name, command }
    }

    pub fn from_convertible<N: Into<SnapName<'a>>, C: Into<App<'b>>>(name: N, command: C) -> Self {
        Self::from_parts(name.into(), command.into())
    }

    pub fn from_raw<'c: 'a + 'b>(raw_command: &'c str) -> Result<Self, SnapdDeserializeError> {
        let (name, command) = raw_command
            .split_once('.')
            .ok_or_else(|| SnapdDeserializeError::MalformedCommand(raw_command.into()))?;

        Ok(Self::from_convertible(name, command))
    }

    pub fn from_raw_owned(raw_command: String) -> Result<Self, SnapdDeserializeError<'a>> {
        // Note: it is *very* important this be `SnapCommand::from_raw` instead of `Self::from_raw`;
        // it took me forever to debug this, but `Self` implies the same lifetimes, which leads to an
        // infinite loop where `Self::from_raw` followed by converting the values into strings forces
        // the compiler to assume that the borrow of `raw_command` from `from_raw`
        // somehow outlives the `'static` lifetime of `String`.
        if let Ok(borrowed) = SnapCommand::from_raw(&raw_command) {
            // There's really no good way to do this without two clones, at least not with severely
            // overcomplicating this struct. We could *technically* have an underlying `raw` that's `Pin`ned
            // and `Box`ed, as well as an `Option`` in case we use `from_parts` to construct this. And then like
            // put `raw_command` on the heap in a pinned location and thus avoid two allocations.
            //
            // But like... why bother?
            return Ok(borrowed.to_owned_inner());
        }

        Err(SnapdDeserializeError::MalformedCommand(raw_command.into()))
    }
}

impl<'a, 'b> ToOwnedInner for SnapCommand<'a, 'b> {
    type Other<'c> = SnapCommand<'c, 'c>;

    fn to_owned_inner<'c>(self) -> Self::Other<'c> {
        SnapCommand {
            name: self.name.to_owned_inner(),
            command: self.command.to_owned_inner(),
        }
    }
}

impl<'a, 'b> Display for SnapCommand<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.name.as_ref(), self.command.as_ref())
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

#[derive(Debug, Clone, Hash, Eq, PartialEq, Error)]
pub enum SnapdDeserializeError<'a> {
    #[error("command string is malformed. expected [name].[command] got {0}.")]
    MalformedCommand(Cow<'a, str>),
}

struct SnapCommandVisitor;

impl<'de> Visitor<'de> for SnapCommandVisitor {
    type Value = SnapCommand<'de, 'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string of the form {name}.{command}")
    }

    fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
        SnapCommand::from_raw(v).map_err(|err| E::custom(err))
    }
}

impl<'de, 'a> Deserialize<'de> for SnapCommand<'a, 'a>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SnapCommandVisitor)
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
