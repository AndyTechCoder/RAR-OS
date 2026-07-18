#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

head=0123456789abcdef0123456789abcdef01234567
merge=89abcdef0123456789abcdef0123456789abcdef
[ "$(tools/toolchain/bind-preauth-head.sh push "$head" - "$head")" = "source_revision=$head" ]
[ "$(tools/toolchain/bind-preauth-head.sh pull_request "$merge" "$head" "$head")" = "source_revision=$head" ]
for rejected in \
    "push $head - $merge" \
    "pull_request $merge $head $merge" \
    "workflow_dispatch $head - $head" \
    "push short - short"; do
    set +e
    # shellcheck disable=SC2086
    tools/toolchain/bind-preauth-head.sh $rejected >/dev/null 2>&1
    status=$?
    set -e
    [ "$status" -eq 73 ] || { echo "source binding rejection failed: $rejected" >&2; exit 1; }
done

test_dir=out/r0/preauth/host-tests
mkdir -p "$test_dir"
oci_test=out/r0/preauth/acquisition/derived-build/host-test
mkdir -p "$oci_test/one" "$oci_test/two"
printf 'deterministic-archive\n' > "$oci_test/one/image.tar"
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
digest=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
printf '{"containerimage.digest": "sha256:%s"}\n' "$digest" > "$oci_test/one/metadata.json"
cp "$oci_test/one/metadata.json" "$oci_test/two/metadata.json"
tools/toolchain/verify-preauth-oci.sh \
    "$oci_test/one/image.tar" "$oci_test/one/metadata.json" \
    "$oci_test/two/image.tar" "$oci_test/two/metadata.json" >/dev/null
printf 'different-archive\n' > "$oci_test/two/image.tar"
set +e
tools/toolchain/verify-preauth-oci.sh \
    "$oci_test/one/image.tar" "$oci_test/one/metadata.json" \
    "$oci_test/two/image.tar" "$oci_test/two/metadata.json" >/dev/null 2>&1
status=$?
set -e
[ "$status" -eq 73 ] || { echo "derived OCI byte mismatch passed" >&2; exit 1; }
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
printf '{"containerimage.digest": "sha256:%064d"}\n' 0 > "$oci_test/two/metadata.json"
set +e
tools/toolchain/verify-preauth-oci.sh \
    "$oci_test/one/image.tar" "$oci_test/one/metadata.json" \
    "$oci_test/two/image.tar" "$oci_test/two/metadata.json" >/dev/null 2>&1
status=$?
set -e
[ "$status" -eq 73 ] || { echo "derived OCI digest mismatch passed" >&2; exit 1; }
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 --test tests/preauth/src/main.rs -o "$test_dir/preauth-tests"
"$test_dir/preauth-tests" --test-threads=1
printf '%s\n' \
    'target_execution=not-attempted' \
    'qemu_execution=not-attempted' \
    'emulator_execution=not-attempted' \
    'vm_execution=not-attempted' \
    'aws_calls=not-attempted'
