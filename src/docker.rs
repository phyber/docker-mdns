// Docker mDNS
use anyhow::Result;
use bollard::container::ListContainersOptions;
use bollard::errors::Error;
use bollard::models::{
    ContainerSummary,
    EventMessage,
};
use bollard::system::EventsOptions;
use crate::mdns;
use futures_core::Stream;
use log::{
    debug,
    info,
};
use std::collections::HashMap;

pub struct Docker {
    conn: bollard::Docker,
}

impl Docker {
    pub fn new() -> Result<Self> {
        let conn = bollard::Docker::connect_with_unix_defaults()?;

        let docker = Self {
            conn: conn,
        };

        Ok(docker)
    }

    pub fn events(&self)
    -> impl Stream<Item = ::std::result::Result<EventMessage, Error>> {
        // We're only interested in container events.
        let filters = HashMap::from([
            ("type".to_string(), vec!["container".to_string()]),
        ]);

        let options = EventsOptions::<String> {
            since: None,
            until: None,
            filters: filters,
        };

        self.conn.events(Some(options))
    }

    pub async fn list_containers(
        &self,
        filters: HashMap<&str, Vec<&str>>,
    )
    -> Result<Vec<ContainerSummary>> {
        let options = ListContainersOptions {
            all: true,
            filters: filters,
            ..Default::default()
        };

        let containers = self.conn
            .list_containers(Some(options))
            .await?;

        Ok(containers)
    }

    // Perform an initial scan of already running containers at startup time
    // so we can setup any required hostnames right away.
    pub async fn startup_scan(&self) -> Result<Vec<mdns::Config>> {
        info!("Performing startup container scan");

        // We want to setup hostnames for any container that's in any kind of
        // "up" state.
        let filters = HashMap::from([
            ("label", vec!["docker-mdns.enable=true"]),
            ("status", vec!["created", "paused", "restarting", "running"]),
        ]);

        let containers = self.list_containers(filters).await?;

        let configs = containers
            .iter()
            .map(mdns::Config::from)
            .collect();

        debug!("Startup container scan found: {:?}", configs);

        Ok(configs)
    }
}
