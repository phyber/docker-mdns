# Docker mDNS

Docker mDNS registers hostnames with Avahi based on labels given to a
container. This allows each container to have a unique `hostname.local`
assigned to it, which should help when reverse proxying from tools like
Traefik or NGINX.

## Minimum Supported Rust Version (MSRV)

The MSRV for this project is currently v1.56.1

## Configuration

The following is an example of a `docker-compose.yaml` file to run Docker mDNS
in a container:

```yaml
---
version: "3"

services:
  mdns:
    container_name: "docker-mdns"
    image: "phyber/docker-mdns:armv7-latest"

    # We need to be able to see the real IP addresses on the host, so we need
    # host networking mode.
    network_mode: 'host'

    # Set this to the name of the interface with the IP addresses you'd like to
    # announce for your hostnames.
    # Both IPv4 and IPv6 addresses will be used.
    command:
      - 'eth0'

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
version: "3"

services:
  traefik:
    container_name: "traefik"
    image: "traefik:v2.5"
    restart: "unless-stopped"
    command:
      - "--providers.docker=true"
      - "--providers.docker.exposebydefault=false"
      - "--entrypoints.web.address=:80"
    labels:
      - "docker-mdns.enable=true"
      - "docker-mdns.host=traefik.local"
      - "traefik.enable=true"
      - "traefik.http.routers.traefik.endpoints=web"
      - "traefik.http.routers.traefik.rule=Host(`traefik.local`)"
      - "traefik.http.services.traefik.loadbalancer.server.port=8080"
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
