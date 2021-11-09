// Docker mDNS
use anyhow::Result;
use bollard::container::ListContainersOptions;
use bollard::errors::Error;
use bollard::models::{
    ContainerSummaryInner,
    SystemEventsResponse,
};
use bollard::system::EventsOptions;
use futures_core::Stream;
use log::{
    debug,
    info,
};
use std::collections::HashMap;
use super::MdnsConfig;

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
    -> impl Stream<Item = ::std::result::Result<SystemEventsResponse, Error>> {
        // We're only interested in container events.
        let filters = HashMap::from([
            ("type".into(), vec!["container".into()]),
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
    -> Result<Vec<ContainerSummaryInner>> {
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
    pub async fn startup_scan(&self) -> Result<Vec<MdnsConfig>> {
        info!("Performing startup container scan");

        let filters = HashMap::from([
            ("label", vec!["docker-mdns.enable=true"]),
            ("status", vec!["running"]),
        ]);

        let containers = self.list_containers(filters).await?;

        let mut hostnames = Vec::new();

        for container in containers {
            let config = MdnsConfig::from(&container.labels);

            hostnames.push(config);
        }

        debug!("Startup container scan found: {:?}", hostnames);

        Ok(hostnames)
    }
}
