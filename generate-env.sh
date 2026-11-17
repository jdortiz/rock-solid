#!/bin/zsh
set -euo pipefail

# Generate an RSA key pair in memory and write it to one dotenv file.
umask 077

ENV_FILE="${1:-.env}"

if ! command -v openssl >/dev/null 2>&1; then
    print -u2 "error: openssl is required"
    exit 1
fi

private_key="$(openssl genpkey \
    -algorithm RSA \
    -pkeyopt rsa_keygen_bits:2048)"

public_key="$(printf '%s\n' "$private_key" | openssl pkey -pubout)"

tmp_file="$(mktemp "${ENV_FILE}.tmp.XXXXXX")"
cleanup() {
    rm -f "$tmp_file"
}
trap cleanup EXIT INT TERM

{
    printf 'JWT_PRIVATE_KEY="%s"\n' "$private_key"
    printf 'JWT_PUBLIC_KEY="%s"\n' "$public_key"
} > "$tmp_file"

chmod 600 "$tmp_file"
mv -f "$tmp_file" "$ENV_FILE"
trap - EXIT INT TERM

print "Generated $ENV_FILE"
