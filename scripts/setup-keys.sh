#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

KEYS_DIR="$DIR/.keys"
PRIV_KEY="$KEYS_DIR/rsa_private.pem"
PUB_KEY="$KEYS_DIR/rsa_public.pem"
TLS_KEY="$KEYS_DIR/dev-tls.key"
TLS_CRT="$KEYS_DIR/dev-tls.crt"

mkdir -p "$KEYS_DIR"

if [[ -f "$PRIV_KEY" && -f "$PUB_KEY" && -f "$TLS_KEY" && -f "$TLS_CRT" && "$1" != "--force" ]]; then
  echo "🔑 Local RSA keypair & TLS certificates already exist in .keys/"
  exit 0
fi

echo "=========================================================="
echo "🔑 Generating custom local 2048-bit RSA keypair & TLS certs..."
echo "=========================================================="

if command -v openssl &> /dev/null; then
  openssl genrsa 2048 2>/dev/null | openssl pkcs8 -topk8 -nocrypt -out "$PRIV_KEY" 2>/dev/null
  openssl rsa -in "$PRIV_KEY" -pubout -out "$PUB_KEY" 2>/dev/null

  # Generate Self-Signed TLS Certificate for astrealabs.local.com & localhost
  openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout "$TLS_KEY" -out "$TLS_CRT" \
    -subj "/CN=astrealabs.local.com/O=Astrea SDA Local Dev" \
    -addext "subjectAltName=DNS:astrealabs.local.com,DNS:localhost,IP:127.0.0.1" 2>/dev/null
else
  python3 -c "
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization

private_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
priv_pem = private_key.private_bytes(
    encoding=serialization.Encoding.PEM,
    format=serialization.PrivateFormat.PKCS8,
    encryption_algorithm=serialization.NoEncryption()
)
pub_pem = private_key.public_key().public_bytes(
    encoding=serialization.Encoding.PEM,
    format=serialization.PublicFormat.SubjectPublicKeyInfo
)

with open('$PRIV_KEY', 'wb') as f:
    f.write(priv_pem)

with open('$PUB_KEY', 'wb') as f:
    f.write(pub_pem)
"
fi

chmod 600 "$PRIV_KEY" "$TLS_KEY"
chmod 644 "$PUB_KEY" "$TLS_CRT"

echo "✅ Created local RSA keypair and TLS certificates successfully:"
echo "   RSA Private Key : $PRIV_KEY"
echo "   RSA Public Key  : $PUB_KEY"
echo "   TLS Certificate : $TLS_CRT"
echo "   TLS Key         : $TLS_KEY"
echo "=========================================================="
