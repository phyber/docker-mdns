// Docker mDNS
use std::collections::HashMap;

const DOCKER_MDNS_ENABLE: &str = "docker-mdns.enable";
const DOCKER_MDNS_HOST: &str = "docker-mdns.host";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MdnsState {
    Disabled,
    Enabled,
}

impl Default for MdnsState {
    fn default() -> Self {
        Self::Disabled
    }
}

impl From<Option<&String>> for MdnsState {
    fn from(s: Option<&String>) -> Self {
        match s.map(String::as_ref) {
            Some("true") => Self::Enabled,
            _            => Self::Disabled,
        }
    }
}

#[derive(Debug, Default)]
pub struct MdnsConfig {
    host: Option<String>,
    state: MdnsState,
}

impl MdnsConfig {
    pub fn enabled(&self) -> bool {
        self.state == MdnsState::Enabled
    }

    pub fn host(&self) -> Option<&String> {
        self.host.as_ref()
    }
}

impl From<&Option<HashMap<String, String>>> for MdnsConfig {
    fn from(attributes: &Option<HashMap<String, String>>) -> Self {
        match attributes {
            None => MdnsConfig::default(),
            Some(attributes) => {
                let enable = attributes.get(DOCKER_MDNS_ENABLE);
                let state = MdnsState::from(enable);
                let host = attributes.get(DOCKER_MDNS_HOST);

                Self {
                    state: state,
                    host: host.cloned(),
                }
            }
        }
    }
}
