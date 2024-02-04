mod alias;
pub(crate) mod snap;

#[doc(inline)]
pub use snap::*;

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
