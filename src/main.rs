// Docker mDNS
#![allow(clippy::redundant_field_names)]
#![forbid(unsafe_code)]
use anyhow::Result;
use bollard::models::SystemEventsResponse;
use futures_util::stream::StreamExt;
use log::{
    debug,
    info,
};
use std::env;

mod bus;
mod docker;
mod mdnsconfig;

use bus::Bus;
use docker::Docker;
use mdnsconfig::MdnsConfig;

#[derive(Debug)]
enum Action {
    Die,
    Start,
}

fn handler<'a>(bus: &mut Bus<'a>, event: &SystemEventsResponse) -> Result<()> {
    debug!("handler event: {:?}", event);

    let action = match event.action.as_ref().map(String::as_ref) {
        Some("die")   => Action::Die,
        Some("start") => Action::Start,
        _             => return Ok(()),
    };

    let actor = match &event.actor {
        Some(actor) => actor,
        None        => return Ok(()),
    };

    let mdns_config = MdnsConfig::from(&actor.attributes);

    match action {
        Action::Die   => bus.unpublish(&mdns_config),
        Action::Start => bus.publish(&mdns_config),
    }
}

fn set_default_log_level() {
    if let Err(_) = env::var("RUST_LOG") {
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
    let mut events = docker.events();

    while let Some(event) = events.next().await {
        let event = event?;
        handler(&mut bus, &event)?;
    }

    Ok(())
}
