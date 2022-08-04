// DBus handling.
// Module is called "bus" to avoid collision with dbus crate.
use anyhow::Result;
use dbus::blocking::Connection;
use dbus::strings::Path;
use if_addrs::get_if_addrs;
use log::{
    debug,
    info,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;
use super::MdnsConfig;

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

pub struct Bus<'a> {
    conn: Connection,
    interface: i32,
    interface_name: String,
    published: HashMap<String, Path<'a>>,
}

impl<'a> Bus<'a> {
    pub fn new(interface: String) -> Result<Self> {
        info!("Getting D-Bus handle for interface: {}", interface);

        let conn = Connection::new_system()?;
        let avahi_interface = Self::avahi_interface(&conn, &interface)?;

        let bus = Self {
            conn: conn,
            interface: avahi_interface,
            interface_name: interface,
            published: HashMap::new(),
        };

        Ok(bus)
    }

    // Gets the avahi interface index.
    // Doesn't act on `self` as we need this number before we're constructed
    // an instance of Self.
    fn avahi_interface(conn: &Connection, interface: &str) -> Result<i32> {
        info!("Getting Avahi Interface number for: {}", interface);

        let proxy = conn.with_proxy(
            NAMESPACE_AVAHI,
            Path::from("/"),
            Duration::from_millis(5_000),
        );

        let (res,): (i32,) = proxy.method_call(
            INTERFACE_SERVER,
            "GetNetworkInterfaceIndexByName",
            (interface,),
        )?;

        debug!("avahi_interface for {} is {}", interface, res);

        Ok(res)
    }

    pub fn publish(&mut self, config: &MdnsConfig) -> Result<()> {
        info!("Publishing config: {:?}", config);

        if !config.enabled() {
            return Ok(());
        }

        let host = match config.host() {
            Some(host) => host.clone(),
            None       => return Ok(()),
        };

        let proxy = self.conn.with_proxy(
            NAMESPACE_AVAHI,
            Path::from("/"),
            Duration::from_millis(5_000),
        );

        // Get a new group to publish under
        let (group_path,): (Path,) = proxy.method_call(
            INTERFACE_SERVER,
            "EntryGroupNew",
            (),
        )?;

        let group = self.conn.with_proxy(
            NAMESPACE_AVAHI,
            &group_path,
            Duration::from_millis(5_000),
        );

        let interface = config.interface().unwrap_or(&self.interface_name);

        // Addresses could change between publishes, so we get them each time.
        let addresses = interface_addresses(interface)?;

        for address in &addresses {
            debug!("AddAddress: {:?}", address);

            group.method_call(
                INTERFACE_ENTRY_GROUP,
                "AddAddress",
                (
                    &self.interface,
                    PROTO_UNSPEC,
                    FLAG_NO_REVERSE,
                    &host,
                    address.to_string(),
                ),
            )?;
        }

        group.method_call(
            INTERFACE_ENTRY_GROUP,
            "Commit",
            (),
        )?;

        debug!("Addresses committed");

        self.published.insert(host, group_path);

        Ok(())
    }

    pub fn unpublish(&mut self, config: &MdnsConfig) -> Result<()> {
        info!("Unpublishing config: {:?}", config);

        let host = match config.host() {
            Some(host) => host.clone(),
            None       => return Ok(()),
        };

        let group_path = match self.published.remove(&host) {
            Some(group_path) => group_path,
            None             => return Ok(()),
        };

        let group = self.conn.with_proxy(
            NAMESPACE_AVAHI,
            &group_path,
            Duration::from_millis(5_000),
        );

        group.method_call(
            INTERFACE_ENTRY_GROUP,
            "Reset",
            (),
        )?;

        group.method_call(
            INTERFACE_ENTRY_GROUP,
            "Free",
            (),
        )?;

        debug!("Unpublished: {}", host);

        Ok(())
    }
}
