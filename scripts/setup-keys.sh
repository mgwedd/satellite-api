#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

KEYS_DIR="$DIR/.keys"
PRIV_KEY="$KEYS_DIR/rsa_private.pem"
PUB_KEY="$KEYS_DIR/rsa_public.pem"

mkdir -p "$KEYS_DIR"

if [[ -f "$PRIV_KEY" && -f "$PUB_KEY" && "$1" != "--force" ]]; then
  echo "🔑 Local RSA keypair already exists in .keys/"
  echo "   Private Key : $PRIV_KEY"
  echo "   Public Key  : $PUB_KEY"
  echo "   (Pass --force to regenerate new keys)"
  exit 0
fi

echo "=========================================================="
echo "🔑 Generating custom local 2048-bit RSA keypair..."
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
echo "   Private Key : $PRIV_KEY"
echo "   Public Key  : $PUB_KEY"
echo ""
echo "Both the API server and ./scripts/make-jwt.sh will automatically"
echo "use your custom local keys for RS256 token signing and verification."
echo "=========================================================="
