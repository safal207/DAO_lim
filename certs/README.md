# DAO TLS Certificates

This directory contains TLS certificates for development and testing.

## Generate Self-Signed Certificates

For development purposes, you can generate self-signed certificates:

```bash
# Generate private key
openssl genrsa -out dao.key 2048

# Generate self-signed certificate (valid for 365 days)
openssl req -new -x509 -key dao.key -out dao.crt -days 365 \
  -subj "/C=US/ST=State/L=City/O=DAO/CN=localhost"
```

Or use the provided script:

```bash
./generate-dev-certs.sh
```

## Production Certificates

For production, use proper certificates from a trusted CA like Let's Encrypt.

**IMPORTANT**: Never commit production certificates to version control!

## Files

- `dao.key` - Private key (git-ignored)
- `dao.crt` - Certificate (git-ignored)
- `.gitignore` - Ensures secrets are not committed
