#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$DIR"

# Ensure local RSA keypair exists in .keys/
"$DIR/scripts/setup-keys.sh" > /dev/null 2>&1

RAW=0
ARGS=()
for arg in "$@"; do
    if [ "$arg" = "--raw" ] || [ "$arg" = "-q" ]; then
        RAW=1
    else
        ARGS+=("$arg")
    fi
done

USERNAME="${ARGS[0]:-admin_user}"
ROLE="${ARGS[1]:-admin}"
TTL_SECONDS="${ARGS[2]:-86400}"

KEYS_DIR="$DIR/.keys"
PRIV_KEY="${LOCAL_DEV_RSA_PRIVATE_KEY_FILE:-${RSA_PRIVATE_KEY_FILE:-$KEYS_DIR/rsa_private.pem}}"

if [ "$RAW" -eq 0 ]; then
    echo "==========================================================" >&2
    echo "🔑 Astrea SDA API - Local RS256 JWT Token Generator" >&2
    echo "==========================================================" >&2
    echo "  Subject   : $USERNAME" >&2
    echo "  Role      : $ROLE" >&2
    echo "  Algorithm : RS256 (RSA 2048-bit Asymmetric)" >&2
    echo "  Key File  : $PRIV_KEY" >&2
    echo "  Lifetime  : ${TTL_SECONDS}s" >&2
    echo "==========================================================" >&2
fi

TOKEN=$(python3 -c "
import json, base64, time, sys
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding

key_path = '$PRIV_KEY'
with open(key_path, 'rb') as f:
    priv_key = serialization.load_pem_private_key(f.read(), password=None)

header = {'alg': 'RS256', 'typ': 'JWT'}
now = int(time.time())
claims = {
    'sub': '$USERNAME',
    'exp': now + $TTL_SECONDS,
    'iat': now,
    'role': '$ROLE'
}

def b64url(data_bytes):
    return base64.urlsafe_b64encode(data_bytes).decode('ascii').rstrip('=')

h_b64 = b64url(json.dumps(header, separators=(',', ':')).encode('utf-8'))
c_b64 = b64url(json.dumps(claims, separators=(',', ':')).encode('utf-8'))
msg = f'{h_b64}.{c_b64}'.encode('ascii')

sig = priv_key.sign(msg, padding.PKCS1v15(), hashes.SHA256())
sig_b64 = b64url(sig)

print(f'{h_b64}.{c_b64}.{sig_b64}')
")

if [ "$RAW" -eq 0 ]; then
    echo "" >&2
    echo "Generated RS256 Bearer Token:" >&2
fi

echo "$TOKEN"

if [ "$RAW" -eq 0 ]; then
    echo "" >&2
    echo "cURL Usage Example:" >&2
    echo "  curl -H \"Authorization: Bearer $TOKEN\" http://localhost:8080/v1/satellites" >&2
    echo "==========================================================" >&2
fi
