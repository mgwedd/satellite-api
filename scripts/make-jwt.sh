#!/usr/bin/env bash
set -e

USERNAME="${1:-admin_user}"
ROLE="${2:-admin}"
SECRET="${JWT_SECRET:-satellite_api_default_jwt_secret_key_change_in_prod}"
TTL_SECONDS="${3:-86400}"

echo "=========================================================="
echo "🔑 Satellite API - Local JWT Token Generator"
echo "=========================================================="
echo "  Subject  : $USERNAME"
echo "  Role     : $ROLE"
echo "  Lifetime : ${TTL_SECONDS}s"
echo "=========================================================="

TOKEN=$(python3 -c "
import json, base64, hmac, hashlib, time, sys

header = {'alg': 'HS256', 'typ': 'JWT'}
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

sig = hmac.new('$SECRET'.encode('utf-8'), msg, hashlib.sha256).digest()
sig_b64 = b64url(sig)

print(f'{h_b64}.{c_b64}.{sig_b64}')
")

echo ""
echo "Generated Bearer Token:"
echo "$TOKEN"
echo ""
echo "cURL Usage Example:"
echo "  curl -H \"Authorization: Bearer $TOKEN\" http://localhost:8080/v1/satellites"
echo "=========================================================="
