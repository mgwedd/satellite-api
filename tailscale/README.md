# 🌐 Tailscale OSS Zero-Config Gateway

This directory contains configuration files for the **Tailscale OSS** containerized gateway in the Astrea SDA API stack across all runtime modes.

---

## 🚀 Overview

Tailscale serves as the **main path** for the "keys included" developer setup workflow with Docker.

### Why Tailscale?
1. **Zero `/etc/hosts` Configuration**: With MagicDNS, the service is accessible by typing `https://sda/` or `https://sda.<tailnet>.ts.net/` on any device on your tailnet.
2. **Real Let's Encrypt TLS Certificates**: Tailscale automatically provisions valid HTTPS certificates. No self-signed certificate warnings, no `-k` / `--insecure` in `curl`.
3. **Short, Easy-to-Type URLs**:
   - `https://sda/` (Interactive Swagger UI)
   - `https://sda/docs` (ReDoc API Reference)
   - `https://sda/v1/...` (Direct API endpoints)
   - `https://sda/api/v1/...` (Subpath prefix alias)
   - `https://sda.dev.astrealabs.com/api/v1/...` (Custom domain alias)
4. **Team Collaboration**: Teammates on your tailnet can access your local running dev or prod test environment directly without deploying to staging or setting up tunnels.

---

## 🔄 Supported Modes

The gateway works across all three container environments:

| Mode | Command | Configuration File | Target Upstream |
| :--- | :--- | :--- | :--- |
| **Host Hot-Reload** | `make dev-hot` | `tailscale/serve-hot.json` | `http://host.docker.internal:8080` |
| **Containerized Dev** | `make dev` | `tailscale/serve-dev.json` | `http://api-dev:8080` |
| **Local Production** | `make prod-run` | `tailscale/serve-prod.json` | `http://api:8080` |

---

## 🔑 Authentication ("Batteries Included")

1. **Option A (Zero-Touch via Key)**:
   Add your Tailscale Auth Key to `.env` (automatically copied from `.env.example` by `make keys`):
   ```bash
   TS_AUTHKEY=tskey-auth-kXXXXX-XXXXX
   ```
   *(Generate reusable or ephemeral auth keys at https://login.tailscale.com/admin/settings/keys)*

2. **Option B (Interactive Approval)**:
   If `TS_AUTHKEY` is omitted, Tailscale generates a one-time interactive browser login link on first container startup.
   Retrieve it instantly with:
   ```bash
   make tailscale-login
   ```
   Or via the CLI helper:
   ```bash
   ./scripts/tailscale.sh login
   ```

---

## 🛠️ CLI Helper Commands

The repository provides `./scripts/tailscale.sh` and corresponding `Makefile` shortcuts:

```bash
make tailscale-status  # Check connection, node name, and MagicDNS status
make tailscale-login   # Display one-time browser login link
make tailscale-urls    # Print all accessible HTTPS and DIY fallback URLs
make tailscale-ping    # Test live connectivity to https://sda/
```

---

## 📁 Configuration Files

- `serve-hot.json`: Reverse-proxies `https://sda/` traffic to `http://host.docker.internal:8080` (`make dev-hot`).
- `serve-dev.json`: Reverse-proxies `https://sda/` traffic to `http://api-dev:8080` (`make dev`).
- `serve-prod.json`: Reverse-proxies `https://sda/` traffic to `http://api:8080` (`make prod-run`).
- `serve.json`: Default fallback configuration.
