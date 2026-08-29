#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
inventory=${1-$root/tools/sprint-alpha/alpha-crypto-references-v1.env}
mode=${2-}
[ -f "$inventory" ] && [ ! -L "$inventory" ] || exit 1
[ "$(/usr/bin/wc -l < "$inventory" | /usr/bin/tr -d ' ')" -eq 14 ] || exit 1
if /usr/bin/grep -Ev '^(schema|state|reference_[12]|version_[12]|license_[12]|provenance_[12]|path_[12]|sha256_[12])=[A-Za-z0-9._:/@+-]+$' "$inventory" | /usr/bin/grep -q .; then exit 1; fi
# shellcheck disable=SC1090
. "$inventory"
[ "$schema" = rar-alpha-crypto-reference-inventory-v1 ] || exit 1
[ "$reference_1|$version_1|$license_1" = 'OpenSSL|3.0.13|Apache-2.0' ] || exit 1
[ "$reference_2|$version_2|$license_2" = 'libsodium|1.0.19|ISC' ] || exit 1
[ "$provenance_1" = 'https://github.com/openssl/openssl/releases/tag/openssl-3.0.13' ] || exit 1
[ "$provenance_2" = 'https://github.com/jedisct1/libsodium/releases/tag/1.0.19-RELEASE' ] || exit 1
[ "$state" = blocked ] || exit 1
[ "$path_1|$sha256_1|$path_2|$sha256_2" = 'unavailable|unavailable|unavailable|unavailable' ] || exit 1
case "$mode" in '' | --require-blocked) ;; *) exit 1 ;; esac
printf 'Alpha crypto reference validation passed: state=%s\n' "$state"
