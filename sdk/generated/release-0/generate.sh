#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)

LC_ALL=C /usr/bin/awk -F '|' '
BEGIN {
    print "//! GENERATED FILE — DO NOT EDIT."
    print "//! Sources: spec/boot/handoff-v1.fields and spec/hardware/rhd-v1.fields."
    print "//! These are owned semantic types. Rust layout is not the wire format."
    print ""
    print "#![no_std]"
    print "#![deny(unsafe_code)]"
    print ""
}
$1 == "rust-const" {
    printf "pub const %s: %s = %s;\n\n", $2, $3, $4
}
$1 == "rust-enum" {
    print "#[derive(Clone, Copy, Debug, Eq, PartialEq)]"
    printf "#[repr(%s)]\n", $3
    printf "pub enum %s {\n", $2
    count = split($4, variants, ",")
    for (i = 1; i <= count; i++) {
        split(variants[i], pair, "=")
        printf "    %s = %s,\n", pair[1], pair[2]
    }
    print "}"
    print ""
}
$1 == "rust-struct" {
    print "#[derive(Clone, Copy, Debug, Eq, PartialEq)]"
    printf "pub struct %s {\n", $2
    count = split($3, fields, ",")
    for (i = 1; i <= count; i++) {
        split(fields[i], pair, ":")
        printf "    pub %s: %s,\n", pair[1], pair[2]
    }
    print "}"
}
' "$root/spec/boot/handoff-v1.fields" "$root/spec/hardware/rhd-v1.fields"
