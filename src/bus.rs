// DBus handling.
use anyhow::Result;
use crate::mdns;
use if_addrs::get_if_addrs;
use std::collections::HashMap;
use std::net::IpAddr;
use tracing::{
    debug,
    info,
};
use zbus::{
    dbus_proxy,
    fdo,
    Connection,
};
use zbus::zvariant::OwnedObjectPath;

const FLAG_NO_REVERSE: u32 = 16;
const PROTO_UNSPEC: i32 = -1;

#[dbus_proxy(
    default_path = "/",
    default_service = "org.freedesktop.Avahi",
    interface = "org.freedesktop.Avahi.Server",
)]
trait AvahiServer {
    // EntryGroupNew
    fn entry_group_new(&self) -> fdo::Result<OwnedObjectPath>;

    // GetNetworkInterfaceIndexByName
    fn get_network_interface_index_by_name(
        &self,
        interface_name: &str,
    ) -> fdo::Result<i32>;
}

#[dbus_proxy(
    default_service = "org.freedesktop.Avahi",
    interface = "org.freedesktop.Avahi.EntryGroup",
)]
trait EntryGroup {
    // AddAddress
    fn add_address(
        &self,
        index: &i32,
        protocol: i32,
        flags: u32,
        host: &str,
        address: String,
    ) -> fdo::Result<()>;

    // Commit
    fn commit(&self) -> fdo::Result<()>;

    // Free
    fn free(&self) -> fdo::Result<()>;

    // Reset
    fn reset(&self) -> fdo::Result<()>;
}

// Returns a Vec of IpAddr for the given interface.
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

pub struct Dbus {
    avahi_interface_index: i32,
    conn: Connection,
    interface_name: String,
    published: HashMap<String, OwnedObjectPath>,
}

impl Dbus {
    pub async fn new(interface_name: String) -> Result<Self> {
        info!("Getting D-Bus handle for interface: {}", interface_name);

        let conn = Connection::system().await?;

        let avahi_interface_index = AvahiServerProxy::new(&conn)
            .await?
            .get_network_interface_index_by_name(&interface_name)
            .await?;

        let dbus = Self {
            avahi_interface_index,
            conn,
            interface_name,
            published: HashMap::new(),
        };

        Ok(dbus)
    }

    pub async fn publish<'a>(&mut self, config: &mdns::Config<'a>) -> Result<()> {
        info!("Publishing config: {:?}", config);

        if !config.enabled() {
            return Ok(());
        }

        let hosts = match config.hosts() {
            Some(hosts) => hosts,
            None        => return Ok(()),
        };

        // Get a new group to publish under
        let proxy = AvahiServerProxy::new(&self.conn).await?;
        let group_path = proxy.entry_group_new().await?;

        let entry_group = EntryGroupProxy::builder(&self.conn)
            .path(&group_path)?
            .build()
            .await?;

        let interface_name = config
            .override_interface()
            .unwrap_or(&self.interface_name);

        // Addresses could change between publishes, so we get them each time.
        let addresses = interface_addresses(interface_name)?;

        for address in &addresses {
            debug!("AddAddress: {:?}", address);

            for host in hosts {
                entry_group.add_address(
                    &self.avahi_interface_index,
                    PROTO_UNSPEC,
                    FLAG_NO_REVERSE,
                    host,
                    address.to_string(),
                ).await?;
            }
        }

        entry_group.commit().await?;

        debug!("Addresses committed");

        self.published.insert(config.id().to_string(), group_path);

        Ok(())
    }

    pub async fn unpublish<'a>(&mut self, config: &mdns::Config<'a>) -> Result<()> {
        info!("Unpublishing config: {:?}", config);

        let id = config.id();

        let group_path = match self.published.remove(id) {
            Some(group_path) => group_path,
            None             => return Ok(()),
        };

        let entry_group = EntryGroupProxy::builder(&self.conn)
            .path(&group_path)?
            .build()
            .await?;

        entry_group.reset().await?;
        entry_group.free().await?;

        debug!("Unpublished: {}", id);

        Ok(())
    }
}
