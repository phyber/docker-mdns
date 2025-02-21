// Docker mDNS
#![forbid(unsafe_code)]
use anyhow::{
    Context,
    Result,
};
use bollard::models::EventMessage;
use futures_util::stream::StreamExt;
use std::env;
use tracing::{
    debug,
    info,
};
use tracing_subscriber::filter::{
    EnvFilter,
    LevelFilter,
};

mod action;
mod bus;
mod docker;
mod mdns;

use action::Action;
use bus::Dbus;
use docker::Docker;

// This is our main processing of the events coming from Docker.
//
// We watch for Die and Start events, and publish or unpublish hostnames based
// on those.
//
// Any other events are ignored.
async fn handler(bus: &mut Dbus, event: &EventMessage) -> Result<()> {
    debug!("handler event: {event:?}");

    // We only deal with Die and Start at the moment. Ignore any Other action.
    let action = match Action::from(&event.action) {
        Action::Other => return Ok(()),
        wanted        => wanted,
    };

    let Some(actor) = &event.actor else {
        return Ok(())
    };

    let mdns_config = mdns::Config::from(actor);

    // Other actions should be unreachable, we already filtered for them above.
    match action {
        Action::Die   => bus.unpublish(&mdns_config).await,
        Action::Start => bus.publish(&mdns_config).await,
        _             => unreachable!(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    let Some(interface) = env::args().nth(1) else {
        eprintln!("Provide an interface to listen on");
        ::std::process::exit(1);
    };

    info!("Interface: {interface:?}");

    // Get a dbus connection
    let mut dbus = Box::pin(Dbus::new(interface))
        .await
        .context("dbus connect")?;

    // Get a docker connection
    let docker = Docker::new()
        .context("docker connect")?;

    // Before entering our main event loop, check if any existing containers
    // need DNS registering.
    let startup_containers = docker
        .startup_scan()
        .await
        .context("startup scan")?;

    for container in startup_containers {
        let config = mdns::Config::from(&container);
        dbus.publish(&config)
            .await
            .context("startup scan publish")?;
    }

    // Now we listen for Docker events
    info!("Entering main Docker events loop");

    let mut events = docker.events();

    while let Some(event) = events.next().await {
        let event = event?;
        handler(&mut dbus, &event)
            .await
            .context("handler")?;
    }

    Ok(())
}
