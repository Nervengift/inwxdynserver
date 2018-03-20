#!/bin/sh
set -e

version="$(grep version Cargo.toml|head|grep -Po '[0-9]+\.[0-9]+\.[0-9]+')"

cargo build --release

fpm -f -s dir -t deb -n inwxdynserver -v "$version" \
	-m dev@nervengiftlabs.de \
	--after-install contrib/install.sh \
	--before-remove contrib/uninstall.sh \
	target/release/inwxdynserver=/usr/bin/ \
	config.example.toml=/etc/inwxdynserver/ \
	contrib/inwxdynserver.service=/lib/systemd/system/
