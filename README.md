# Robotics Deployment Embedded

![embedded](rdembedded.png)

[![Build](https://github.com/Robotics-Deployment/embedded/actions/workflows/build.yml/badge.svg)](https://github.com/Robotics-Deployment/embedded/actions/workflows/build.yml)

### Build

```bash
docker compose build
```

### About

---

The embedded code that runs on remote devices. 

This module: 

- Self validates the configuration
- If the configuration is incomplete, it will fetch it
- Configures its wireguard VPN
- Sends a heartbeat to the server
