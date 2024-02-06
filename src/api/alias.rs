use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{snap_str_newtype, App, SnapCommand, SnapName};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum AliasCommand<'a> {
    Alias {
        #[serde(borrow)]
        snap: SnapName<'a>,
        #[serde(borrow)]
        alias: SnapAlias<'a>,
        #[serde(borrow, skip_serializing_if = "Option::is_none")]
        app: Option<App<'a>>,
    },
    Unalias {
        #[serde(borrow, skip_serializing_if = "Option::is_none")]
        snap: Option<SnapName<'a>>,
        #[serde(borrow)]
        alias: SnapAlias<'a>,
        #[serde(borrow, skip_serializing_if = "Option::is_none")]
        app: Option<App<'a>>,
    },
    Prefer {
        #[serde(borrow)]
        snap: SnapName<'a>,
        #[serde(borrow)]
        alias: SnapAlias<'a>,
        #[serde(borrow, skip_serializing_if = "Option::is_none")]
        app: Option<App<'a>>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Aliases<'a>(
    #[serde(borrow)] HashMap<SnapName<'a>, HashMap<SnapAlias<'a>, AliasInfo<'a>>>,
);

#[derive(Clone, Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct AliasInfo<'a> {
    #[serde(borrow)]
    command: SnapCommand<'a, 'a>,
    #[serde(flatten, borrow)]
    status: AliasStatus<'a>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum AliasStatus<'a> {
    Auto {
        #[serde(rename = "auto", borrow)]
        app_name: App<'a>,
    },
    Manual {
        #[serde(rename = "manual", borrow)]
        app_name: App<'a>,
    },
    Disabled,
}

snap_str_newtype! {
    SnapAlias
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn deserialize_info() {
        let json = r#"
        {
            "snap":
            {
                "alias1":
                {
                    "command": "snap.app",
                    "status": "auto",
                    "auto": "app"
                },
                "alias2":
                {
                    "command": "foo",
                    "status": "manual",
                    "manual": "app1"
                }
            }
        }
        "#;

        let mut alias_map = HashMap::with_capacity(2);
        alias_map.insert(
            "alias1".into(),
            AliasInfo {
                command: SnapCommand::from_convertible("snap", "app"),
                status: AliasStatus::Auto {
                    app_name: "app".into(),
                },
            },
        );

        alias_map.insert(
            "alias2".into(),
            AliasInfo {
                command: SnapCommand::from_raw("foo"),
                status: AliasStatus::Manual {
                    app_name: "app1".into(),
                },
            },
        );

        let mut snap_map = HashMap::with_capacity(1);
        snap_map.insert("snap".into(), alias_map);

        let aliases = Aliases(snap_map);

        assert_eq!(
            aliases,
            serde_json::from_str(json).expect("could not decode alias response json")
        )
    }

    #[test]
    fn serialize_command_no_app() {
        let command = AliasCommand::Alias {
            snap: "steam".into(),
            alias: "games".into(),
            app: None,
        };

        let expected = r#"{"action":"alias","snap":"steam","alias":"games"}"#;

        assert_eq!(
            serde_json::to_string(&command).expect("could not serialize"),
            expected
        );
    }

    #[test]
    fn serialize_command_app() {
        let command = AliasCommand::Alias {
            snap: "steam".into(),
            alias: "vulkan".into(),
            app: Some("vkinfo".into()),
        };

        let expected = r#"{"action":"alias","snap":"steam","alias":"vulkan","app":"vkinfo"}"#;

        assert_eq!(
            serde_json::to_string(&command).expect("could not serialize"),
            expected
        );
    }
}
