# ⚡ Caddy Zero-Config OSS Gateway

This directory contains configuration files for the **Caddy** (Apache 2.0) containerized gateway in the Astrea SDA API stack.

---

## 🚀 Overview

Caddy is the **100% open-source, zero-account, batteries-included** gateway for local development and testing.

### Why Caddy?
1. **100% Open Source**: Apache 2.0 license, written in Go. No proprietary SaaS control planes, no accounts, and no tokens.
2. **Zero `/etc/hosts` Configuration**: Uses `sda.localtest.me` (or `localhost`). `*.localtest.me` is a globally registered public domain whose DNS records permanently resolve to `127.0.0.1`.
3. **Automatic Local HTTPS**: Built-in Certificate Authority (`tls internal`). Generates and rotates TLS certificates on the fly without running OpenSSL or wrestling with self-signed certificate generation scripts.
4. **Convenient URLs**:
   - `https://sda.localtest.me:8443` (Interactive Swagger UI)
   - `https://sda.localtest.me:8443/docs` (ReDoc Interactive Docs)
   - `https://sda.localtest.me:8443/v1/...` (Direct API Endpoints)
   - `https://sda.localtest.me:8443/sda/api/v1/...` (Subpath prefix alias)
   - `http://localhost:8888` (HTTP fallback)
   - `http://localhost:8880` (Direct container port)

---

## 🔄 Supported Modes

| Mode | Command | Caddy Configuration | Target Upstream |
| :--- | :--- | :--- | :--- |
| **Host Hot-Reload** | `make dev-hot` | `caddy/Caddyfile.dev-hot` | `http://host.docker.internal:8080` |
| **Containerized Dev** | `make dev` | `caddy/Caddyfile` | `http://api:8080` (via `api-dev` alias) |
| **Local Production** | `make prod-run` | `caddy/Caddyfile` | `http://api:8080` |

---

## 📁 Files

- `Caddyfile`: Reverse-proxies traffic to `http://api:8080` (used by `make dev` and `make prod-run`).
- `Caddyfile.dev-hot`: Reverse-proxies traffic to `http://host.docker.internal:8080` (used by `make dev-hot`).
