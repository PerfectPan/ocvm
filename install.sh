#!/bin/sh
set -eu

REPO="${OCVM_REPO:-PerfectPan/ocvm}"
INSTALL_DIR="${OCVM_INSTALL_DIR:-$HOME/.local/bin}"
API_BASE="${GITHUB_API_URL:-https://api.github.com}"

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

need curl
need tar

system="$(uname -s)"
machine="$(uname -m)"

case "$system" in
  Darwin) os="apple-darwin" ;;
  Linux) os="unknown-linux-gnu" ;;
  *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac

case "$machine" in
  x86_64 | amd64) arch="x86_64" ;;
  arm64 | aarch64)
    if [ "$system" = "Darwin" ]; then
      arch="aarch64"
    else
      echo "unsupported CPU architecture for Linux release assets: $machine" >&2
      exit 1
    fi
    ;;
  *) echo "unsupported CPU architecture: $(uname -m)" >&2; exit 1 ;;
esac

target="${arch}-${os}"
asset="ocvm-${target}.tar.gz"
tmp="${TMPDIR:-/tmp}/ocvm-install.$$"
mkdir -p "$tmp"
trap 'rm -rf "$tmp"' EXIT INT TERM

auth_header=""
if [ -n "${GITHUB_TOKEN:-}" ]; then
  auth_header="Authorization: Bearer ${GITHUB_TOKEN}"
fi

api_url="${API_BASE}/repos/${REPO}/releases/latest"
release_json="$tmp/release.json"

if [ -n "$auth_header" ]; then
  curl -fsSL -H "$auth_header" "$api_url" -o "$release_json"
else
  curl -fsSL "$api_url" -o "$release_json"
fi

download_url="$(sed -n "s/.*\"browser_download_url\": \"\\([^\"]*${asset}\\)\".*/\\1/p" "$release_json" | head -n 1)"
checksum_url="$(sed -n "s/.*\"browser_download_url\": \"\\([^\"]*${asset}.sha256\\)\".*/\\1/p" "$release_json" | head -n 1)"

if [ -z "$download_url" ]; then
  echo "could not find release asset ${asset} in ${REPO}" >&2
  exit 1
fi

if [ -n "$auth_header" ]; then
  curl -fsSL -H "$auth_header" "$download_url" -o "$tmp/$asset"
  if [ -n "$checksum_url" ]; then
    curl -fsSL -H "$auth_header" "$checksum_url" -o "$tmp/$asset.sha256"
  fi
else
  curl -fsSL "$download_url" -o "$tmp/$asset"
  if [ -n "$checksum_url" ]; then
    curl -fsSL "$checksum_url" -o "$tmp/$asset.sha256"
  fi
fi

if [ -s "$tmp/$asset.sha256" ]; then
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$tmp" && sha256sum -c "$asset.sha256")
  elif command -v shasum >/dev/null 2>&1; then
    expected="$(awk '{print $1}' "$tmp/$asset.sha256")"
    actual="$(shasum -a 256 "$tmp/$asset" | awk '{print $1}')"
    [ "$expected" = "$actual" ] || {
      echo "checksum mismatch for $asset" >&2
      exit 1
    }
  else
    echo "warning: checksum available but sha256sum/shasum is missing" >&2
  fi
fi

mkdir -p "$INSTALL_DIR"
tar -C "$tmp" -xzf "$tmp/$asset"
cp "$tmp/ocvm-${target}/ocvm" "$INSTALL_DIR/ocvm"
chmod +x "$INSTALL_DIR/ocvm"

cat <<EOF
ocvm installed to ${INSTALL_DIR}/ocvm

Add these to your shell profile if they are not already present:

  export PATH="${INSTALL_DIR}:\$PATH"
  export PATH="\$HOME/.ocvm/shims:\$PATH"

For shell helpers, run:

  ocvm init zsh
  ocvm init bash
  ocvm init fish
EOF
