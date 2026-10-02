#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

KEYS_DIR="$DIR/.keys"
PRIV_KEY="$KEYS_DIR/rsa_private.pem"
PUB_KEY="$KEYS_DIR/rsa_public.pem"

mkdir -p "$KEYS_DIR"

# Clean up legacy Nginx self-signed certificate files if present
rm -f "$KEYS_DIR/dev-tls.crt" "$KEYS_DIR/dev-tls.key"

# Initialize .env from template if missing
if [ ! -f "$DIR/.env" ] && [ -f "$DIR/.env.example" ]; then
  cp "$DIR/.env.example" "$DIR/.env"
  echo "📄 Initialized .env configuration file from .env.example"
fi

if [[ -f "$PRIV_KEY" && -f "$PUB_KEY" && "$1" != "--force" ]]; then
  echo "🔑 Local RSA 2048 keypair already exists in .keys/"
  exit 0
fi

echo "=========================================================="
echo "🔑 Generating custom local 2048-bit RSA keypair for JWT auth..."
echo "=========================================================="

if command -v openssl &> /dev/null; then
  openssl genrsa 2048 2>/dev/null | openssl pkcs8 -topk8 -nocrypt -out "$PRIV_KEY" 2>/dev/null
  openssl rsa -in "$PRIV_KEY" -pubout -out "$PUB_KEY" 2>/dev/null
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

chmod 600 "$PRIV_KEY"
chmod 644 "$PUB_KEY"

echo "✅ Created local RSA keypair successfully:"
echo "   RSA Private Key : $PRIV_KEY"
echo "   RSA Public Key  : $PUB_KEY"
echo "   (TLS certificates are automatically managed by Caddy)"
echo "=========================================================="
