// DBus handling.
// Module is called "bus" to avoid collision with dbus crate.
use anyhow::Result;
use dbus::blocking::Connection;
use dbus::strings::Path;
use if_addrs::get_if_addrs;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;
use super::MdnsConfig;

const FLAG_NO_REVERSE: u32 = 16;
const INTERFACE_ENTRY_GROUP: &str = "org.freedesktop.Avahi.EntryGroup";
const INTERFACE_SERVER: &str = "org.freedesktop.Avahi.Server";
const NAMESPACE_AVAHI: &str = "org.freedesktop.Avahi";
const PROTO_UNSPEC: i32 = -1;

fn avahi_interface(conn: &Connection, interface: &str) -> Result<i32> {
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

    println!("Interface: {:?}", res);

    Ok(res)
}

fn interface_address(interface: &str) -> Result<Vec<IpAddr>> {
    let addrs: Vec<IpAddr> = get_if_addrs()?
        .into_iter()
        .filter(|i| {
            !i.addr.is_loopback() && i.name == interface
        })
        .map(|i| i.ip())
        .collect();

    println!("{:#?}", addrs);

    Ok(addrs)
}

pub struct Bus<'a> {
    conn: Connection,
    addresses: Vec<IpAddr>,
    interface: i32,
    interface_name: String,
    published: HashMap<String, Path<'a>>,
}

impl<'a> Bus<'a> {
    pub fn new(interface: String) -> Result<Self> {
        let conn = Connection::new_system()?;
        let addresses = interface_address(&interface)?;
        let avahi_interface = avahi_interface(&conn, &interface)?;

        let bus = Self {
            conn: conn,
            addresses: addresses,
            interface: avahi_interface,
            interface_name: interface,
            published: HashMap::new(),
        };

        Ok(bus)
    }

    pub fn publish(&mut self, config: &MdnsConfig) -> Result<()> {
        println!("Publishing host: {:?}", config);

        if !config.enabled() {
            return Ok(());
        }

        let host = match config.host() {
            Some(host) => host.to_owned(),
            None       => return Ok(()),
        };

        let proxy = self.conn.with_proxy(
            NAMESPACE_AVAHI,
            Path::from("/"),
            Duration::from_millis(5_000),
        );

        println!("Before EntryGroupNew");

        // Get a new group to publish under
        let (group_path,): (Path,) = proxy.method_call(
            INTERFACE_SERVER,
            "EntryGroupNew",
            (),
        )?;

        println!("AfterMethod: {:?}", group_path);

        let group = self.conn.with_proxy(
            NAMESPACE_AVAHI,
            &group_path,
            Duration::from_millis(5_000),
        );

        for address in &self.addresses {
            println!("Adding address: {:?}", address);

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

        println!("Published for host: {}", host);

        self.published.insert(host, group_path);

        Ok(())
    }

    pub fn unpublish(&mut self, config: &MdnsConfig) -> Result<()> {
        println!("Unpublishing host: {:?}", config);

        let host = match config.host() {
            Some(host) => host.to_owned(),
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

        println!("Unpublished: {}", host);

        Ok(())
    }
}
