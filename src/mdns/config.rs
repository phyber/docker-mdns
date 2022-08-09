// Docker mDNS
use bollard::models::{
    ContainerSummary,
    EventActor,
};
use crate::mdns::State;

// Docker labels that we're interested in.
const DOCKER_MDNS_ENABLE: &str = "docker-mdns.enable";
const DOCKER_MDNS_HOST: &str = "docker-mdns.host";
const DOCKER_MDNS_INTERFACE: &str = "docker-mdns.interface";

#[derive(Debug, Default, PartialEq)]
pub struct Config {
    // Hosts taken from docker-mdns.host
    // These are the hostnames to be announced via Avahi.
    hosts: Option<Vec<String>>,

    // The container ID that this Config is for.
    id: String,

    // The override interface provided via docker-mdns.interface, if any.
    interface: Option<String>,

    // The state, Enabled or Disabled. Taken from docker-mdns.enable.
    state: State,
}

// A basic impl that exposes some methods instead of allowing other code
// direct access to struct members.
impl Config {
    pub fn enabled(&self) -> bool {
        self.state == State::Enabled
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

// Takes an EventActor from Docker and turns it into an appropriate Config.
impl From<&EventActor> for Config {
    fn from(eventactor: &EventActor) -> Self {
        // The events that we're interested in should always come with a
        // container ID.
        let id = match &eventactor.id {
            Some(id) => id.clone(),
            None     => panic!("Expected actor id"),
        };

        match &eventactor.attributes {
            None => {
                // Basic Disabled configuration if we don't get any attributes
                // (labels).
                Self {
                    id: id,
                    ..Self::default()
                }
            },
            Some(attributes) => {
                // We got attributes (labels), look harder to see if there's
                // work to do.
                let enable = attributes.get(DOCKER_MDNS_ENABLE);
                let hosts = attributes.get(DOCKER_MDNS_HOST);
                let interface = attributes.get(DOCKER_MDNS_INTERFACE);
                let state = State::from(enable);

                // Build a vec of hosts from the string we get from the label.
                // These are the hostnames that will be published.
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
// over again, we quickly make an EventActor and call the above from() based on
// that.
impl From<&ContainerSummary> for Config {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // This test also indirectly tests From<&EventActor>
    #[test]
    fn test_from_container_summary() {
        let id = "abc123".to_string();
        let labels = HashMap::from([
            ("docker-mdns.enable".to_string(), "false".to_string()),
        ]);

        let input = ContainerSummary {
            id: Some(id.clone()),
            labels: Some(labels),
            ..Default::default()
        };

        let config = Config::from(&input);

        let expected = Config {
            id: id.clone(),
            ..Default::default()
        };

        assert_eq!(config, expected);
    }
}
