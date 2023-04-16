// Docker mDNS
use bollard::models::{
    ContainerSummary,
    EventActor,
};
use crate::mdns::State;
use std::borrow::Cow;
use std::convert::AsRef;

// Docker labels that we're interested in.
const DOCKER_MDNS_ENABLE: &str = "docker-mdns.enable";
const DOCKER_MDNS_HOST: &str = "docker-mdns.host";
const DOCKER_MDNS_INTERFACE: &str = "docker-mdns.interface";

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Config<'a> {
    // Hosts taken from docker-mdns.host
    // These are the hostnames to be announced via Avahi.
    hosts: Option<Vec<Cow<'a, str>>>,

    // The container ID that this Config is for.
    id: Cow<'a, str>,

    // The override interface provided via docker-mdns.interface, if any.
    override_interface: Option<Cow<'a, str>>,

    // The state, Enabled or Disabled. Taken from docker-mdns.enable.
    state: State,
}

// A basic impl that exposes some methods instead of allowing other code
// direct access to struct members.
impl<'a> Config<'a> {
    pub fn enabled(&self) -> bool {
        self.state == State::Enabled
    }

    pub fn hosts(&self) -> Option<Vec<&str>> {
        self.hosts.as_ref().map(|v| v.iter().map(AsRef::as_ref).collect())
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn override_interface(&self) -> Option<&str> {
        self.override_interface.as_ref().map(AsRef::as_ref)
    }
}

// Takes an EventActor from Docker and turns it into an appropriate Config.
impl<'a> From<&'a EventActor> for Config<'a> {
    fn from(eventactor: &'a EventActor) -> Self {
        // The events that we're interested in should always come with a
        // container ID.
        let id = match &eventactor.id {
            Some(id) => id,
            None     => panic!("Expected actor id"),
        };

        match &eventactor.attributes {
            None => {
                // Basic Disabled configuration if we don't get any attributes
                // (labels).
                Self {
                    id: id.into(),
                    ..Self::default()
                }
            },
            Some(attributes) => {
                // We got attributes (labels), look harder to see if there's
                // work to do.
                let enable = attributes.get(DOCKER_MDNS_ENABLE);
                let hosts = attributes.get(DOCKER_MDNS_HOST);
                let override_interface = attributes.get(DOCKER_MDNS_INTERFACE)
                    .map(Cow::from);
                let state = State::from(enable);

                // Build a vec of hosts from the string we get from the label.
                // These are the hostnames that will be published.
                let hosts = hosts
                    .map(|hosts| {
                        hosts
                            .split_whitespace()
                            .map(Cow::from)
                            .collect::<Vec<Cow<'a, str>>>()
                    });

                Self {
                    hosts: hosts,
                    id: id.into(),
                    override_interface: override_interface,
                    state: state,
                }
            }
        }
    }
}

// This is called from docker.rs. Instead of implementing the above logic all
// over again, we quickly make an EventActor and call the above from() based on
// that.
impl<'a> From<&'a ContainerSummary> for Config<'a> {
    fn from(summary: &'a ContainerSummary) -> Self {
        let id = match &summary.id {
            Some(id) => id,
            None     => panic!("expected container id"),
        };

        // Summary labels are the same as EventActor attributes
        match &summary.labels {
            None => {
                // Basic Disabled configuration if we don't get any attributes
                // (labels).
                Self {
                    id: id.into(),
                    ..Self::default()
                }
            },
            Some(attributes) => {
                // We got attributes (labels), look harder to see if there's
                // work to do.
                let enable = attributes.get(DOCKER_MDNS_ENABLE);
                let hosts = attributes.get(DOCKER_MDNS_HOST);
                let override_interface = attributes.get(DOCKER_MDNS_INTERFACE)
                    .map(Cow::from);
                let state = State::from(enable);

                // Build a vec of hosts from the string we get from the label.
                // These are the hostnames that will be published.
                let hosts = hosts
                    .map(|hosts| {
                        hosts
                            .split_whitespace()
                            .map(Cow::from)
                            .collect::<Vec<Cow<'a, str>>>()
                    });

                Self {
                    hosts: hosts,
                    id: id.into(),
                    override_interface: override_interface,
                    state: state,
                }
            }
        }
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
            id: id.into(),
            ..Default::default()
        };

        assert_eq!(config, expected);
    }
}
