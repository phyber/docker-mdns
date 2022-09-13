// DBus handling.
// Module is called "bus" to avoid collision with dbus crate.
use anyhow::Result;
use crate::mdns;
use if_addrs::get_if_addrs;
use log::{
    debug,
    info,
};
use std::collections::HashMap;
use std::net::IpAddr;
use zbus::{
    Connection,
    Proxy,
};
use zbus::zvariant::OwnedObjectPath;

const FLAG_NO_REVERSE: u32 = 16;
const INTERFACE_ENTRY_GROUP: &str = "org.freedesktop.Avahi.EntryGroup";
const INTERFACE_SERVER: &str = "org.freedesktop.Avahi.Server";
const NAMESPACE_AVAHI: &str = "org.freedesktop.Avahi";
const PROTO_UNSPEC: i32 = -1;

// Returns a Vec of IpAddr for the given interface.
//
// We only call this once when getting our Bus handle, but we should probably
// call it each time we publish, as interface addresses may change and listing
// IP addresses should be a fairly cheap operation.
fn interface_addresses(interface: &str) -> Result<Vec<IpAddr>> {
    info!("Getting interface addresses for {}", interface);

    let addrs: Vec<IpAddr> = get_if_addrs()?
        .into_iter()
        .filter(|i| !i.addr.is_loopback() && i.name == interface)
        .map(|i| i.ip())
        .collect();

    debug!("{:?}", addrs);

    Ok(addrs)
}

pub struct Bus {
    avahi_interface_index: i32,
    conn: Connection,
    interface_name: String,
    published: HashMap<String, OwnedObjectPath>,
}

impl Bus {
    pub async fn new(interface_name: String) -> Result<Self> {
        info!("Getting D-Bus handle for interface: {}", interface_name);

        let conn = Connection::system().await?;

        let avahi_interface_index = Self::avahi_interface_index_from_name(
            &conn,
            &interface_name,
        ).await?;

        let bus = Self {
            avahi_interface_index: avahi_interface_index,
            conn: conn,
            interface_name: interface_name,
            published: HashMap::new(),
        };

        Ok(bus)
    }

    // Gets the avahi interface index.
    // Doesn't act on `self` as we need this number before we're constructed
    // an instance of Self.
    async fn avahi_interface_index_from_name(
        conn: &Connection,
        interface_name: &str,
    ) -> Result<i32> {
        info!("Getting Avahi Interface Index for: {}", interface_name);

        let reply = conn.call_method(
            Some(NAMESPACE_AVAHI),
            "/",
            Some(INTERFACE_SERVER),
            "GetNetworkInterfaceIndexByName",
            &(interface_name,),
        ).await?;

        let index: i32 = reply.body()?;

        debug!("avahi_interface for {} is {}", interface_name, index);

        Ok(index)
    }

    pub async fn publish(&mut self, config: &mdns::Config) -> Result<()> {
        info!("Publishing config: {:?}", config);

        if !config.enabled() {
            return Ok(());
        }

        let hosts = match config.hosts() {
            Some(hosts) => hosts,
            None        => return Ok(()),
        };

        // Get a new group to publish under
        let proxy = Proxy::new(
            &self.conn,
            NAMESPACE_AVAHI,
            "/",
            INTERFACE_SERVER,
        ).await?;

        let group_path: OwnedObjectPath = proxy.call(
            "EntryGroupNew",
            &(),
        ).await?;

        let entry_group = Proxy::new(
            &self.conn,
            NAMESPACE_AVAHI,
            &group_path,
            INTERFACE_ENTRY_GROUP,
        ).await?;

        let interface_name = config
            .override_interface()
            .unwrap_or(&self.interface_name);

        // Addresses could change between publishes, so we get them each time.
        let addresses = interface_addresses(interface_name)?;

        for address in &addresses {
            debug!("AddAddress: {:?}", address);

            for host in hosts {
                entry_group.call_method(
                    "AddAddress",
                    &(
                        &self.avahi_interface_index,
                        PROTO_UNSPEC,
                        FLAG_NO_REVERSE,
                        &host,
                        address.to_string(),
                    ),
                ).await?;
            }
        }

        entry_group.call_method("Commit", &()).await?;

        debug!("Addresses committed");

        self.published.insert(config.id().to_string(), group_path);

        Ok(())
    }

    pub async fn unpublish(&mut self, config: &mdns::Config) -> Result<()> {
        info!("Unpublishing config: {:?}", config);

        let id = config.id();

        let group_path = match self.published.remove(id) {
            Some(group_path) => group_path,
            None             => return Ok(()),
        };

        let entry_group = Proxy::new(
            &self.conn,
            NAMESPACE_AVAHI,
            &group_path,
            INTERFACE_ENTRY_GROUP,
        ).await?;

        entry_group.call_method("Reset", &()).await?;
        entry_group.call_method("Free", &()).await?;

        debug!("Unpublished: {}", id);

        Ok(())
    }
}
