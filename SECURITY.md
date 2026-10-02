# Security Policy

## Reporting a Vulnerability

The Astrea SDA API team takes security seriously. If you discover a security vulnerability in this repository, please do **NOT** open a public GitHub issue.

Instead, please report the vulnerability privately by contacting the security team at:
**[michael@astrealabs.com]**

Please include:
1. Description of the vulnerability and potential impact.
2. Step-by-step instructions or proof-of-concept script to reproduce the issue.
3. Any proposed mitigations or fixes.

We will acknowledge receipt of your report within 48 hours and provide updates on resolution status.

---

## Security Practices & Core Guidelines

### 1. RSA Key Isolation & Development Secrets
- **Local Keypair Isolation**: `make install-dev` generates local RSA keypairs inside `.keys/` strictly for local testing. `.keys/` is listed in `.gitignore` and must NEVER be committed to Git.
- **Production Secrets**: In production environments, RSA PEM private/public keys MUST be injected strictly via environment variables (`RSA_PRIVATE_KEY` and `RSA_PUBLIC_KEY`) via secret managers (e.g. HashiCorp Vault, AWS Secrets Manager, Doppler).

### 2. JWT RS256 Authentication
- All write/mutation endpoints (`POST`, `PUT`, `DELETE`) require a valid RS256 JWT Bearer token signed by the trusted authority.
- HS256 / RS256 algorithm confusion attacks are explicitly checked and rejected by the authentication layer ([`src/auth/mod.rs`](src/auth/mod.rs)).

### 3. Rate Limiting & Compute Shielding
- Heavy astrodynamics compute endpoints are protected by static complexity evaluation ($O(1)$ Gatekeeper), global concurrency semaphores, and per-user identity quotas (`DashMap<String, Arc<Semaphore>>`).
