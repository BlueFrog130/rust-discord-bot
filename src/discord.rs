use bitflags::bitflags;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum Interaction {
    Ping,
    ApplicationCommand {
        data: ApplicationCommandData,
        interaction: InteractionCommon,
    },
}

impl<'de> Deserialize<'de> for Interaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let t = value.get("type").and_then(Value::as_u64).unwrap();

        Ok(match t {
            1 => Interaction::Ping,
            2 => {
                let data = value.get("data").unwrap();

                Interaction::ApplicationCommand {
                    data: ApplicationCommandData::deserialize(data).unwrap(),
                    interaction: InteractionCommon::deserialize(value).unwrap(),
                }
            }
            _ => panic!("Unknown type"),
        })
    }
}

pub enum InteractionResponse<'a> {
    ChannelMessageWithSource(ChannelMessageWithSourceData<'a>),
    Pong,
}

#[derive(Copy, Clone)]
enum InteractionResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
    DeferredUpdateMessage = 6,
    UpdateMessage = 7,
    ApplicationCommandAutocompleteResult = 8,
    Modal = 9,
}

impl Serialize for InteractionResponse<'static> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            InteractionResponse::Pong => {
                let mut state = serializer.serialize_struct("", 1)?;
                state.serialize_field("type", &(InteractionResponseType::Pong as u8))?;
                state.end()
            }
            InteractionResponse::ChannelMessageWithSource(data) => {
                let mut state = serializer.serialize_struct("", 2)?;
                state.serialize_field(
                    "type",
                    &(InteractionResponseType::ChannelMessageWithSource as u8),
                )?;
                state.serialize_field("data", data)?;
                state.end()
            }
            _ => panic!("Unhandled type"),
        }
    }
}

#[derive(Serialize)]
pub struct ChannelMessageWithSourceData<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<()>,
}

impl<'a> ChannelMessageWithSourceData<'a> {
    pub fn content(content: &'a str) -> Self {
        ChannelMessageWithSourceData {
            content: Some(content),
            tts: None,
            embeds: None,
            allowed_mentions: None,
            flags: None,
            components: None,
            attachments: None,
        }
    }
}

bitflags! {
    pub struct MessageFlags: u32 {
        const CROSSPOSTED = 1 << 0;
        const IS_CROSSPOST = 1 << 1;
        const SUPPRESS_EMBEDS = 1 << 2;
        const SOURCE_MESSAGE_DELETED = 1 << 3;
        const URGENT = 1 << 4;
        const HAS_THREAD = 1 << 5;
        const EPHEMERAL = 1 << 6;
        const LOADING = 1 << 7;
        const FAILED_TO_MENTION_SOME_ROLES_IN_THREAD = 1 << 8;
    }
}

#[derive(Serialize)]
pub struct Embed {}

#[derive(Deserialize, Debug)]
pub struct InteractionCommon {
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub token: String,
    pub id: String,
    pub application_id: String,
}

#[derive(Deserialize, Debug)]
pub struct ApplicationCommandData {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub t: u8,
    pub options: Option<Vec<ApplicationCommandOption>>,
}

#[derive(Debug)]
pub struct ApplicationCommandOption {
    pub name: String,
    pub value: ApplicationCommandOptionType,
}

impl<'de> Deserialize<'de> for ApplicationCommandOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let name = value.get("name").and_then(Value::as_str).unwrap();

        Ok(ApplicationCommandOption {
            name: name.to_string(),
            value: ApplicationCommandOptionType::deserialize(value).unwrap(),
        })
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ApplicationCommandOptionType {
    SUB_COMMAND,
    SUB_COMMAND_GROUP,
    STRING(String),
    INTEGER(i32),
    BOOLEAN(bool),
    USER,
    CHANNEL,
    ROLE,
    MENTIONABLE,
    NUMBER(f64),
    ATTACHMENT,
}

impl<'de> Deserialize<'de> for ApplicationCommandOptionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        println!("{}", value.to_string());
        let t = value.get("type").and_then(Value::as_u64).unwrap();
        let data = value.get("value").unwrap();

        Ok(match t {
            1 => ApplicationCommandOptionType::SUB_COMMAND,
            2 => ApplicationCommandOptionType::SUB_COMMAND_GROUP,
            3 => ApplicationCommandOptionType::STRING(String::deserialize(data).unwrap()),
            4 => ApplicationCommandOptionType::INTEGER(i32::deserialize(data).unwrap()),
            5 => ApplicationCommandOptionType::BOOLEAN(bool::deserialize(data).unwrap()),
            6 => ApplicationCommandOptionType::USER,
            7 => ApplicationCommandOptionType::CHANNEL,
            8 => ApplicationCommandOptionType::ROLE,
            9 => ApplicationCommandOptionType::MENTIONABLE,
            10 => ApplicationCommandOptionType::NUMBER(f64::deserialize(data).unwrap()),
            11 => ApplicationCommandOptionType::ATTACHMENT,
            _ => panic!("Unknown type"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes() {
        let data = r#"
        {
            "type": 2,
            "token": "A_UNIQUE_TOKEN",
            "member": {
                "user": {
                    "id": "53908232506183680",
                    "username": "Mason",
                    "avatar": "a_d5efa99b3eeaa7dd43acca82f5692432",
                    "discriminator": "1337",
                    "public_flags": 131141
                },
                "roles": ["539082325061836999"],
                "premium_since": null,
                "permissions": "2147483647",
                "pending": false,
                "nick": null,
                "mute": false,
                "joined_at": "2017-03-13T19:19:14.040000+00:00",
                "is_pending": false,
                "deaf": false
            },
            "id": "786008729715212338",
            "guild_id": "290926798626357999",
            "app_permissions": "442368",
            "application_id": "",
            "guild_locale": "en-US",
            "locale": "en-US",
            "data": {
                "options": [{
                    "type": 3,
                    "name": "cardname",
                    "value": "The Gitrog Monster"
                }],
                "type": 1,
                "name": "cardsearch",
                "id": "771825006014889984"
            },
            "channel_id": "645027906669510667"
        }
        "#;

        let _msg: Interaction = serde_json::from_str(data).unwrap();

        assert!(true);
    }

    #[test]
    fn deserializes_ping() {
        let req = r#"
        {
            "type": 1
        }
        "#;

        let _msg: Interaction = serde_json::from_str(req).unwrap();

        assert!(true);
    }

    #[test]
    fn serializes() {
        let response =
            InteractionResponse::ChannelMessageWithSource(ChannelMessageWithSourceData {
                tts: None,
                content: Some("Test"),
                embeds: None,
                allowed_mentions: None,
                flags: None,
                components: None,
                attachments: None,
            });

        let resp = serde_json::to_string_pretty(&response).unwrap();

        println!("{resp}");

        assert!(true);
    }

    #[test]
    fn serializes_ping() {
        let response = InteractionResponse::Pong;

        let resp = serde_json::to_string_pretty(&response).unwrap();

        println!("{resp}");

        assert!(true);
    }
}
