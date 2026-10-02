# 🌐 Tailscale OSS Zero-Config Gateway

This directory contains configuration files for the **Tailscale OSS** containerized gateway in the Astrea SDA API developer stack.

---

## 🚀 Overview

Tailscale serves as the **main path** for the "keys included" developer setup workflow with Docker.

### Why Tailscale?
1. **Zero `/etc/hosts` Configuration**: With MagicDNS, the service is accessible by typing `https://sda/` or `https://sda.<tailnet>.ts.net/` on any device on your tailnet.
2. **Real Let's Encrypt TLS Certificates**: Tailscale automatically provisions valid HTTPS certificates. No self-signed certificate warnings, no `-k` / `--insecure` in `curl`.
3. **Short, Easy-to-Type URLs**:
   - `https://sda/` (Interactive Swagger UI / ReDoc)
   - `https://sda/v1/...` (Direct API endpoints)
   - `https://sda/api/v1/...` (Subpath prefix alias)
   - `https://sda.dev.astrealabs.com/api/v1/...` (Custom domain alias)
4. **Team Collaboration**: Teammates on your tailnet can access your local running dev environment directly without deploying to staging or using tunnels.

---

## 🔑 Authentication ("Keys Included")

1. Add your Tailscale Auth Key to `.env` (or pass as environment variable):
   ```bash
   TS_AUTHKEY=tskey-auth-kXXXXX-XXXXX
   ```
   *(Generate reusable or ephemeral auth keys at https://login.tailscale.com/admin/settings/keys)*

2. If `TS_AUTHKEY` is omitted, Tailscale will print an interactive one-time browser login link on first container startup (`docker compose logs tailscale` or `make tailscale-login`).

---

## 📁 Files

- `serve.json`: Reverse-proxies `https://sda/` traffic to `http://api-dev:8080` (used by `make dev` / `docker-compose.dev.yml`).
- `serve-hot.json`: Reverse-proxies `https://sda/` traffic to `http://host.docker.internal:8080` (used by `make dev-hot` / `docker-compose.dev-hot.yml`).
