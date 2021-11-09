# Docker Avahi

Docker Avahi registers hostnames with Avahi based on labels given to a
container. This allows each container to have a unique `hostname.local`
assigned to it, which should help when reverse proxying from tools like
Traefik or NGINX.

## Configuration

The following is an example of a `docker-compose.yaml` file configuring a
container to use Docker Avahi:

```yaml
---
version: "3"

services:
  mdns:
    container_name: "docker-mdns"
    image: "phyber/docker-mdns:v1.0"
    volumes:
      - "/var/run/dbus/system_bus_socket:/var/run/dbus/system_bus_socket:rw"
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
