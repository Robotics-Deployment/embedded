# Robotics Deployment Embedded


[![Build](https://github.com/Robotics-Deployment/embedded/actions/workflows/build.yml/badge.svg)](https://github.com/Robotics-Deployment/embedded/actions/workflows/build.yml)

### Build

```bash
docker compose build
```

### About

---

The embedded code that runs on nodes.

This module: 
- Validates the configuration of the system
- Looks up the IP of the hub on wireguard 
- Sends a heartbeat to the hub
