#!/bin/sh
set -eu

root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
install_script="$root/install.sh"
site_installer_dir="$root/site/public/ocvm"

mkdir -p "$site_installer_dir"
cp "$install_script" "$site_installer_dir/install.sh"

echo "synced site installer asset"
