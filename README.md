# Docker mDNS

Docker mDNS registers hostnames with Avahi based on labels given to a
container. This allows each container to have a unique `hostname.local`
assigned to it, which should help when reverse proxying from tools like
Traefik or NGINX.

## Minimum Supported Rust Version (MSRV)

The MSRV for this project is currently v1.81.0

## Configuration

Docker mDNS takes a single command line argument which is the host interface
that it should use to find addresses to announce.  For example, if you launch
it as `docker-mdns eth0` and `eth0` has the IP address `10.20.30.40` this will
be the IP address used when when replying to mDNS queries.

Docker mDNS will attempt to find all IP (v4 and v6) addresses on an interface,
as long as they aren't loopback addresses.

The rest of the Docker mDNS configuration is done through labels. The supported
labels are:

| Label                   | Default | Description                     |
|-------------------------|---------|---------------------------------|
| `docker-mdns.enable`    | `false` | Enable mDNS for the container   |
| `docker-mdns.host`      | `None`  | Hostname(s) to announce         |
| `docker-mdns.interface` | `None`  | Interface to get addresses from |

The `docker-mdns.host` label can take a list of whitespace separated hostnames
if you want multiple hostnames for a container.

Providing `docker-mdns.interface` will override which interface Docker mDNS
uses when finding IP addresses to announce for the `docker-mdns.host`.

The following is an example of a `docker-compose.yaml` file to run Docker mDNS
in a container:

```yaml
---
version: "3.8"

services:
  mdns:
    container_name: "docker-mdns"
    image: "ghcr.io/phyber/docker-mdns:aarch64-latest"

    # We need to be able to see the real IP addresses on the host, so we need
    # host networking mode.
    network_mode: "host"

    # Set this to the name of the interface with the IP addresses you'd like to
    # announce for your hostnames.
    # Both IPv4 and IPv6 addresses will be used.
    command:
      - "eth0"

    # We need to be able to read docker.sock to watch for container events,
    # and we need to be able to write to the system d-bus socket to add
    # hostnames in Avahi
    volumes:
      - "/var/run/dbus/system_bus_socket:/var/run/dbus/system_bus_socket:rw"
      - "/var/run/docker.sock:/var/run/docker.sock:ro"
```

And the following would configure Traefik to present its dashboard on a
configured mDNS hostname:

```yaml
---
version: "3.8"

services:
  traefik:
    container_name: "traefik"
    image: "traefik:v2.8.1"
    restart: "unless-stopped"
    command:
      - "--entrypoints.web.address=:80"
      - "--providers.docker=true"
      - "--providers.docker.exposebydefault=false"
    labels:
      docker-mdns.enable: "true"
      docker-mdns.host: "traefik.local"
      traefik.enable: "true"
      traefik.http.routers.traefik.endpoints: "web"
      traefik.http.routers.traefik.rule: "Host(`traefik.local`)"
      traefik.http.services.traefik.loadbalancer.server.port: "8080"
    ports:
      - "80:80"
      - "8080:8080"
    volumes:
      - "/var/run/docker.sock:/var/run/docker.sock:ro"
```

## License

Licensed under either of

  * Apache License, Version 2.0
    ([LICENSE-APACHE] or https://www.apache.org/licenses/LICENSE-2.0)
  * MIT license
    ([LICENSE-MIT] or https://opensource.org/licenses/MIT)

at your option.

<!-- links -->
[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT
