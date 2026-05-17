#!/bin/sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$root"

expect_output() {
  expected="$1"
  shift
  output="$("$@")"
  printf '%s\n' "$output" | grep -F "$expected" >/dev/null || {
    echo "expected output to contain: $expected" >&2
    echo "$output" >&2
    exit 1
  }
}

expect_file_text() {
  file="$1"
  text="$2"
  grep -F "$text" "$file" >/dev/null || {
    echo "expected $file to contain: $text" >&2
    exit 1
  }
}

expect_files_equal() {
  left="$1"
  right="$2"
  cmp -s "$left" "$right" || {
    echo "expected $left and $right to match" >&2
    exit 1
  }
}

expect_output "asset=ocvm-x86_64-unknown-linux-gnu.tar.gz" \
  env OCVM_INSTALL_DRY_RUN=1 OCVM_TEST_UNAME_S=Linux OCVM_TEST_UNAME_M=x86_64 ./install.sh

expect_output "asset=ocvm-x86_64-apple-darwin.tar.gz" \
  env OCVM_INSTALL_DRY_RUN=1 OCVM_TEST_UNAME_S=Darwin OCVM_TEST_UNAME_M=x86_64 ./install.sh

expect_output "asset=ocvm-aarch64-apple-darwin.tar.gz" \
  env OCVM_INSTALL_DRY_RUN=1 OCVM_TEST_UNAME_S=Darwin OCVM_TEST_UNAME_M=arm64 ./install.sh

expect_output "api_url=https://api.github.com/repos/PerfectPan/ocvm/releases/tags/v0.1.1" \
  env OCVM_INSTALL_DRY_RUN=1 OCVM_VERSION=v0.1.1 OCVM_TEST_UNAME_S=Darwin OCVM_TEST_UNAME_M=arm64 ./install.sh

./scripts/sync-site-installer.sh >/dev/null

expect_file_text .github/workflows/release.yml "target: x86_64-unknown-linux-gnu"
expect_file_text .github/workflows/release.yml "target: x86_64-apple-darwin"
expect_file_text .github/workflows/release.yml "target: aarch64-apple-darwin"
expect_file_text .github/workflows/release.yml "target: x86_64-pc-windows-msvc"
expect_file_text .github/workflows/release.yml 'shasum -a 256 "${name}.tar.gz" > "${name}.tar.gz.sha256"'
expect_file_text .github/workflows/release.yml '"$hash  $name.zip"'
expect_files_equal install.sh site/public/ocvm/install.sh

echo "release prep checks passed"
