#!/bin/sh
set -eu

image="${OCVM_E2E_IMAGE:-ocvm-e2e:local}"

docker build -f tests/e2e/Dockerfile -t "$image" .
docker run --rm "$image"

