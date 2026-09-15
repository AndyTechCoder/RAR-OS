#!/bin/sh
# Historical mistaken registration path; retained to honor no-deletion policy.
# This file is not referenced by any workflow and must never launch validation.
# The sole authoritative entry is tools/ci/check-alpha-crypto-primitives.sh.
printf '%s\n' 'Refused: use the reviewed tools/ci validation entrypoint.' >&2
exit 2
