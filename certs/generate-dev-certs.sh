#!/bin/bash
# Generate self-signed certificates for DAO development

set -e

CERT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CERT_DIR"

echo "Generating DAO development certificates..."

# Generate private key
openssl genrsa -out dao.key 2048
echo "✓ Private key generated: dao.key"

# Generate self-signed certificate
openssl req -new -x509 -key dao.key -out dao.crt -days 365 \
  -subj "/C=US/ST=Liminal/L=Gateway/O=DAO/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,DNS:*.localhost,IP:127.0.0.1"

echo "✓ Certificate generated: dao.crt"
echo ""
echo "Certificates are ready for development use."
echo "Valid for 365 days for: localhost, *.localhost, 127.0.0.1"
echo ""
echo "Update your dao.toml to use these certificates:"
echo "  tls_cert = \"certs/dao.crt\""
echo "  tls_key  = \"certs/dao.key\""
