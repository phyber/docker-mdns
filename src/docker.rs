// Docker mDNS
use anyhow::{
    Context,
    Result,
};
use bollard::errors::Error;
use bollard::models::{
    ContainerSummary,
    EventMessage,
};
use bollard::query_parameters::{
    EventsOptions,
    ListContainersOptions,
};
use futures_core::Stream;
use std::collections::HashMap;
use tracing::{
    debug,
    info,
};

pub struct Docker {
    conn: bollard::Docker,
}

impl Docker {
    pub fn new() -> Result<Self> {
        let conn = bollard::Docker::connect_with_unix_defaults()?;

        let docker = Self {
            conn,
        };

        Ok(docker)
    }

    pub fn events(&self)
    -> impl Stream<Item = ::std::result::Result<EventMessage, Error>> {
        // We're only interested in container events.
        let filters = HashMap::from([
            (
                String::from("action"),
                vec![
                    String::from("die"),
                    String::from("start"),
                ],
            ),
            (
                String::from("type"),
                vec![
                    String::from("container"),
                ],
            ),
        ]);

        let options = EventsOptions {
            filters: Some(filters),
            since: None,
            until: None,
        };

        self.conn.events(Some(options))
    }

    pub async fn list_containers(
        &self,
        filters: Option<HashMap<String, Vec<String>>>,
    )
    -> Result<Vec<ContainerSummary>> {
        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = self.conn
            .list_containers(Some(options))
            .await
            .context("list containers")?;

        Ok(containers)
    }

    // Perform an initial scan of already running containers at startup time
    // so we can setup any required hostnames right away.
    pub async fn startup_scan(&self) -> Result<Vec<ContainerSummary>> {
        info!("Performing startup container scan");

        // We want to setup hostnames for any container that's in any kind of
        // "up" state.
        let filters = HashMap::from([
            (
                String::from("label"),
                vec![
                    String::from("docker-mdns.enable=true"),
                ],
            ),
            (
                String::from("status"),
                vec![
                    String::from("created"),
                    String::from("paused"),
                    String::from("restarting"),
                    String::from("running"),
                ],
            ),
        ]);

        let containers = self.list_containers(Some(filters)).await?;

        debug!("Startup container scan found: {containers:?}");

        Ok(containers)
    }
}
