use serde::{Deserialize, Serialize};

use super::{is_not, ClientRoles, InvocationPolicy, MatchingPolicy, RouterRoles, URI};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct HelloDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    roles: ClientRoles,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct WelcomeDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    roles: RouterRoles,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct ErrorDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct SubscribeOptions {
    #[serde(
        default,
        rename = "match",
        skip_serializing_if = "MatchingPolicy::is_strict"
    )]
    pub pattern_match: MatchingPolicy,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct PublishOptions {
    #[serde(default, skip_serializing_if = "is_not")]
    acknowledge: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct RegisterOptions {
    #[serde(
        default,
        rename = "match",
        skip_serializing_if = "MatchingPolicy::is_strict"
    )]
    pub pattern_match: MatchingPolicy,

    #[serde(
        default,
        rename = "invoke",
        skip_serializing_if = "InvocationPolicy::is_single"
    )]
    pub invocation_policy: InvocationPolicy,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct CallOptions {}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct YieldOptions {}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct EventDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    publisher: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    trustlevel: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<URI>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct InvocationDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub procedure: Option<URI>,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ResultDetails {}

impl HelloDetails {
    pub fn new(roles: ClientRoles) -> HelloDetails {
        HelloDetails { roles, agent: None }
    }

    pub fn new_with_agent(roles: ClientRoles, agent: &str) -> HelloDetails {
        HelloDetails {
            roles,
            agent: Some(agent.to_string()),
        }
    }
}

impl WelcomeDetails {
    pub fn new(roles: RouterRoles) -> WelcomeDetails {
        WelcomeDetails { roles, agent: None }
    }

    pub fn new_with_agent(roles: RouterRoles, agent: &str) -> WelcomeDetails {
        WelcomeDetails {
            roles,
            agent: Some(agent.to_string()),
        }
    }
}

impl ErrorDetails {
    pub fn new() -> ErrorDetails {
        ErrorDetails { message: None }
    }

    pub fn new_with_message(message: &str) -> ErrorDetails {
        ErrorDetails {
            message: Some(message.to_string()),
        }
    }
}

impl SubscribeOptions {
    pub fn new() -> SubscribeOptions {
        SubscribeOptions {
            pattern_match: MatchingPolicy::Strict,
        }
    }
}

impl PublishOptions {
    pub fn new(acknowledge: bool) -> PublishOptions {
        PublishOptions { acknowledge }
    }

    pub fn should_acknowledge(&self) -> bool {
        self.acknowledge
    }
}

impl RegisterOptions {
    pub fn new() -> RegisterOptions {
        RegisterOptions {
            pattern_match: MatchingPolicy::Strict,
            invocation_policy: InvocationPolicy::Single,
        }
    }
}

impl CallOptions {
    pub fn new() -> CallOptions {
        CallOptions {}
    }
}

impl YieldOptions {
    pub fn new() -> YieldOptions {
        YieldOptions {}
    }
}

impl EventDetails {
    pub fn new() -> EventDetails {
        EventDetails {
            publisher: None,
            trustlevel: None,
            topic: None,
        }
    }

    pub fn new_with_topic(topic: URI) -> EventDetails {
        EventDetails {
            publisher: None,
            trustlevel: None,
            topic: Some(topic),
        }
    }
}

impl InvocationDetails {
    pub fn new() -> InvocationDetails {
        InvocationDetails { procedure: None }
    }
}

impl ResultDetails {
    pub fn new() -> ResultDetails {
        ResultDetails {}
    }
}
