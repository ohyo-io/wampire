use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::is_not;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ClientRoles {
    pub publisher: PublisherRole,
    pub subscriber: SubscriberRole,
    pub caller: CallerRole,
    pub callee: CalleeRole,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RouterRoles {
    pub dealer: DealerRole,
    pub broker: BrokerRole,
}

/**************************
          Roles
**************************/
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PublisherRole {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    features: Option<HashMap<String, bool>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct CallerRole {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    features: Option<HashMap<String, bool>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct CalleeRole {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    features: Option<HashMap<String, bool>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SubscriberRole {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    features: Option<SubscriberFeatures>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SubscriberFeatures {
    #[serde(skip_serializing_if = "is_not", default)]
    pattern_based_subscription: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DealerRole {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    features: Option<DealerFeatures>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BrokerRole {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    features: Option<BrokerFeatures>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DealerFeatures {
    #[serde(skip_serializing_if = "is_not", default)]
    pattern_based_registration: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct BrokerFeatures {
    #[serde(skip_serializing_if = "is_not", default)]
    pattern_based_subscription: bool,
}

/**************************
      Implementations
**************************/

impl RouterRoles {
    #[inline]
    pub fn new() -> RouterRoles {
        RouterRoles {
            broker: BrokerRole {
                features: Some(BrokerFeatures {
                    pattern_based_subscription: true,
                }),
            },
            dealer: DealerRole {
                features: Some(DealerFeatures {
                    pattern_based_registration: true,
                }),
            },
        }
    }

    #[inline]
    pub fn new_basic() -> RouterRoles {
        RouterRoles {
            broker: BrokerRole { features: None },
            dealer: DealerRole { features: None },
        }
    }
}

impl ClientRoles {
    #[inline]
    pub fn new() -> ClientRoles {
        ClientRoles {
            publisher: PublisherRole {
                features: Some(HashMap::new()),
            },
            subscriber: SubscriberRole {
                features: Some(SubscriberFeatures {
                    pattern_based_subscription: true,
                }),
            },
            caller: CallerRole {
                features: Some(HashMap::new()),
            },
            callee: CalleeRole {
                features: Some(HashMap::new()),
            },
        }
    }

    #[inline]
    pub fn new_basic() -> ClientRoles {
        ClientRoles {
            publisher: PublisherRole {
                features: Some(HashMap::new()),
            },
            subscriber: SubscriberRole {
                features: Some(SubscriberFeatures {
                    pattern_based_subscription: false,
                }),
            },
            caller: CallerRole {
                features: Some(HashMap::new()),
            },
            callee: CalleeRole {
                features: Some(HashMap::new()),
            },
        }
    }
}

impl Default for RouterRoles {
    fn default() -> RouterRoles {
        RouterRoles::new()
    }
}

impl Default for ClientRoles {
    fn default() -> ClientRoles {
        ClientRoles::new()
    }
}
