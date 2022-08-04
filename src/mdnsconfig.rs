// Docker mDNS
use std::collections::HashMap;

const DOCKER_MDNS_ENABLE: &str = "docker-mdns.enable";
const DOCKER_MDNS_HOST: &str = "docker-mdns.host";
const DOCKER_MDNS_INTERFACE: &str = "docker-mdns.interface";

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
    interface: Option<String>,
    state: MdnsState,
}

impl MdnsConfig {
    pub fn enabled(&self) -> bool {
        self.state == MdnsState::Enabled
    }

    pub fn host(&self) -> Option<&String> {
        self.host.as_ref()
    }

    pub fn interface(&self) -> Option<&String> {
        self.interface.as_ref()
    }
}

impl From<&Option<HashMap<String, String>>> for MdnsConfig {
    fn from(attributes: &Option<HashMap<String, String>>) -> Self {
        match attributes {
            None => MdnsConfig::default(),
            Some(attributes) => {
                let enable = attributes.get(DOCKER_MDNS_ENABLE);
                let host = attributes.get(DOCKER_MDNS_HOST);
                let interface = attributes.get(DOCKER_MDNS_INTERFACE);
                let state = MdnsState::from(enable);

                Self {
                    host: host.cloned(),
                    interface: interface.cloned(),
                    state: state,
                }
            }
        }
    }
}
