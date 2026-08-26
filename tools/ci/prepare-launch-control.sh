#!/bin/sh
set -eu

control=${1-}
[ "$(/usr/bin/uname -s)" = Linux ] || exit 1
[ "${GITHUB_ACTIONS-}" = true ] && [ "${CI-}" = true ] || exit 1
[ -d "$control" ] && [ ! -L "$control" ] || exit 1
[ -z "$(find "$control" -mindepth 1 -maxdepth 1 ! -name '._*' -print -quit)" ] || exit 1
/bin/mkdir "$control/to-host" "$control/to-launch"
[ ! -L "$control/to-host" ] && [ ! -L "$control/to-launch" ] || exit 1
/bin/chmod 0711 "$control"
/bin/chmod 0733 "$control/to-host"
/bin/chmod 0755 "$control/to-launch"
