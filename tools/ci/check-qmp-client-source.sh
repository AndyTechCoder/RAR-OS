#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
tree=${1-$root/tools/rar-lab/qmp-client}
[ -d "$tree" ] && [ ! -L "$tree" ] || exit 1
expected='README.md
build-plan.v1
json.rs
main.rs'
actual=$(find "$tree" -mindepth 1 -maxdepth 1 ! -name '._*' -print | /usr/bin/sed "s|^$tree/||" | /usr/bin/sort)
[ "$actual" = "$expected" ] || exit 1
find "$tree" -type l -print | /usr/bin/grep -q . && exit 1

plan=$tree/build-plan.v1
[ "$(/usr/bin/wc -l < "$plan" | /usr/bin/tr -d ' ')" -eq 16 ] || exit 1
for line in \
    'schema=rar-qmp-build-plan-v1' \
    'owner=RAR' \
    'language=Rust' \
    'edition=2024' \
    'target=x86_64-unknown-linux-musl' \
    'crate_name=rar_qmp_client' \
    'entry=main.rs' \
    'module=json.rs' \
    'dependency_policy=rust-std-only' \
    'compiler=/opt/rar-toolchain/bin/rustc' \
    'test_output=/build/rar-qmp-client-tests' \
    'binary_output=/build/rar-qmp-client' \
    'install_path=/opt/rar-lab/bin/rar-qmp-client' \
    'network=none'; do
    [ "$(/usr/bin/grep -Fxc "$line" "$plan")" -eq 1 ] || exit 1
done
/usr/bin/grep -Fqx 'test_args=--edition=2024,--test,-C,debuginfo=0,--remap-path-prefix=/controller=.' "$plan" || exit 1
/usr/bin/grep -Fqx 'release_args=--edition=2024,-C,opt-level=s,-C,debuginfo=0,-C,strip=symbols,-C,codegen-units=1,-C,panic=abort,-C,metadata=rar_qmp_client_v1,--remap-path-prefix=/controller=.,--target=x86_64-unknown-linux-musl' "$plan" || exit 1

for source in "$tree/main.rs" "$tree/json.rs"; do
    [ -s "$source" ] && [ ! -L "$source" ] || exit 1
    ! /usr/bin/grep -En '(unsafe[[:space:]]+(fn|impl|extern)|unsafe[[:space:]]*\{|extern[[:space:]]+crate|include!|include_bytes!|include_str!|todo!|unimplemented!|std::process::Command|Command::new|TcpStream|UdpSocket|libc::)' "$source" >/dev/null || exit 1
done
/usr/bin/grep -Fqx 'const SOCKET: &str = "/tmp/rar-qmp.sock";' "$tree/main.rs" || exit 1
/usr/bin/grep -Fqx 'const SERIAL: &str = "/evidence/serial.log";' "$tree/main.rs" || exit 1
for verb in wait-ready continue key-chord pointer serial-offset wait-trace capture quit; do
    /usr/bin/grep -Fq "\"$verb\"" "$tree/main.rs" || exit 1
done
for source in docs/interop/qmp-spec.txt qapi/misc.json qapi/ui.json; do
    /usr/bin/grep -Fq "https://gitlab.com/qemu-project/qemu/-/raw/b67b00e6b4c7831a3f5bc684bc0df7a9bfd1bd56/$source" "$tree/README.md" || exit 1
done
/usr/bin/grep -Fq '0x7fff' "$tree/main.rs" || exit 1
/usr/bin/grep -Fq 'fs::hard_link(&temporary, &path)' "$tree/main.rs" || exit 1
/usr/bin/grep -Fq 'QMP success payload is not an empty object' "$tree/main.rs" || exit 1
printf '%s\n' 'QMP client source policy passed'
