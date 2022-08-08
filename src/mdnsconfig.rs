// Docker mDNS
use bollard::models::{
    ContainerSummary,
    EventActor,
};

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
    hosts: Option<Vec<String>>,
    id: String,
    interface: Option<String>,
    state: MdnsState,
}

impl MdnsConfig {
    pub fn enabled(&self) -> bool {
        self.state == MdnsState::Enabled
    }

    pub fn hosts(&self) -> &Option<Vec<String>> {
        &self.hosts
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn interface(&self) -> Option<&String> {
        self.interface.as_ref()
    }
}

impl From<&EventActor> for MdnsConfig {
    fn from(eventactor: &EventActor) -> Self {
        // We should always get a container ID on start or die
        let id = match &eventactor.id {
            Some(id) => id.clone(),
            None     => panic!("Expected actor id"),
        };

        match &eventactor.attributes {
            None => {
                Self {
                    id: id,
                    ..Self::default()
                }
            },
            Some(attributes) => {
                let enable = attributes.get(DOCKER_MDNS_ENABLE);
                let hosts = attributes.get(DOCKER_MDNS_HOST);
                let interface = attributes.get(DOCKER_MDNS_INTERFACE);
                let state = MdnsState::from(enable);

                // Build a vec of hosts from the string we get from the label.
                let hosts = hosts
                    .map(|hosts| {
                        hosts
                            .split_whitespace()
                            .map(String::from)
                            .collect::<Vec<String>>()
                    });

                Self {
                    hosts: hosts,
                    id: id,
                    interface: interface.cloned(),
                    state: state,
                }
            }
        }
    }
}

// This is called from docker.rs. Instead of implementing the above logic all
// over again, we quickly make an EventActor and call the above from based on
// that.
impl From<&ContainerSummary> for MdnsConfig {
    fn from(summary: &ContainerSummary) -> Self {
        let id = match &summary.id {
            Some(id) => Some(id.clone()),
            None     => panic!("expected container id"),
        };

        let actor = EventActor {
            attributes: summary.labels.clone(),
            id: id,
        };

        Self::from(&actor)
    }
}
