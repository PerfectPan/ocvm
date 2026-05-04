#!/bin/sh
set -eu

bin="${OCVM_BIN:-./target/release/ocvm}"
home="${OCVM_HOME:-/tmp/ocvm-home}"

rm -rf "$home"
mkdir -p "$home"

"$bin" list-remote --channel latest
"$bin" install latest
"$bin" default latest
"$bin" current
"$bin" exec -- openclaw --version
"$bin" doctor || true

echo "e2e npm openclaw test completed with OCVM_HOME=$home"

