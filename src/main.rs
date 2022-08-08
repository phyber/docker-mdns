// Docker mDNS
#![allow(clippy::redundant_field_names)]
#![forbid(unsafe_code)]
use anyhow::Result;
use bollard::models::EventMessage;
use futures_util::stream::StreamExt;
use log::{
    debug,
    info,
};
use std::env;

mod action;
mod bus;
mod docker;
mod mdnsconfig;

use action::Action;
use bus::Bus;
use docker::Docker;
use mdnsconfig::MdnsConfig;

// This is our main processing of the events coming from Docker.
//
// We watch for Die and Start events, and publish or unpublish hostnames based
// on those.
//
// Any other events are ignored.
fn handler<'a>(bus: &mut Bus<'a>, event: &EventMessage) -> Result<()> {
    debug!("handler event: {:?}", event);

    // We only deal with Die and Start at the moment. Ignore any Other action.
    let action = match Action::from(&event.action) {
        Action::Other => return Ok(()),
        wanted        => wanted,
    };

    let actor = match &event.actor {
        Some(actor) => actor,
        None        => return Ok(()),
    };

    let mdns_config = MdnsConfig::from(actor);

    // Other actions should be unreachable, we already filtered for them above.
    match action {
        Action::Die   => bus.unpublish(&mdns_config),
        Action::Start => bus.publish(&mdns_config),
        _             => unreachable!(),
    }
}

fn set_default_log_level() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "docker_mdns=info");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    set_default_log_level();
    pretty_env_logger::init();

    let interface = match env::args().nth(1) {
        Some(interface) => interface,
        None            => {
            eprintln!("Provide an interface to listen on");
            ::std::process::exit(1);
        }
    };

    info!("Interface: {:?}", interface);

    // Get a dbus connection
    let mut bus = Bus::new(interface)?;

    // Get a docker connection
    let docker = Docker::new()?;

    // Before entering our main event loop, check if any existing containers
    // need DNS registering.
    let startup_configs = docker.startup_scan().await?;
    for config in startup_configs {
        bus.publish(&config)?;
    }

    // Now we listen for Docker events
    info!("Entering main Docker events loop");

    let mut events = docker.events();

    while let Some(event) = events.next().await {
        let event = event?;
        handler(&mut bus, &event)?;
    }

    Ok(())
}
