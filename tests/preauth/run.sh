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
mkdir -p out/r0/preauth/host-tools
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 tools/toolchain/preauth-verify-oci.rs \
    -o out/r0/preauth/host-tools/preauth-verify-oci
oci_test=out/r0/preauth/acquisition/derived-build/host-test
mkdir -p "$oci_test/root/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    "$oci_test/one" "$oci_test/two"
printf 'minimal deterministic layer\n' > "$oci_test/root/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/layer.tar"
layer_digest=$(/usr/bin/sha256sum "$oci_test/root/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/layer.tar" | /usr/bin/cut -d ' ' -f 1)
printf '{"rootfs":{"type":"layers","diff_ids":["sha256:%s"]}}\n' "$layer_digest" > "$oci_test/root/config.pending"
digest=$(/usr/bin/sha256sum "$oci_test/root/config.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/config.pending" "$oci_test/root/$digest.json"
printf '[{"Config":"%s.json","RepoTags":["fixture:latest"],"Layers":["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/layer.tar"]}]\n' "$digest" > "$oci_test/root/manifest.json"
printf '{}\n' > "$oci_test/root/repositories"
printf '1.0\n' > "$oci_test/root/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/VERSION"
printf '{}\n' > "$oci_test/root/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/json"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/one/image.tar" -C "$oci_test/root" \
    manifest.json repositories "$digest.json" \
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/VERSION \
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/json \
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/layer.tar
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
printf '{"containerimage.config.digest":"sha256:%s","containerimage.digest":"sha256:%s"}\n' "$digest" "$digest" > "$oci_test/one/metadata.json"
cp "$oci_test/one/metadata.json" "$oci_test/two/metadata.json"
printf 'sha256:%s\n' "$digest" > "$oci_test/one/image.id"
cp "$oci_test/one/image.id" "$oci_test/two/image.id"
tools/toolchain/verify-preauth-oci.sh \
    "$oci_test/one/image.tar" "$oci_test/one/metadata.json" "$oci_test/one/image.id" \
    "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" >/dev/null
expect_oci_rejection() {
    description=$1
    set +e
    tools/toolchain/verify-preauth-oci.sh \
        "$oci_test/one/image.tar" "$oci_test/one/metadata.json" "$oci_test/one/image.id" \
        "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" >/dev/null 2>&1
    status=$?
    set -e
    [ "$status" -eq 73 ] || { echo "$description passed strict OCI verification" >&2; exit 1; }
}
printf 'different-archive\n' > "$oci_test/two/image.tar"
expect_oci_rejection "derived OCI byte mismatch"
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
printf '{"containerimage.digest": "sha256:%064d"}\n' 0 > "$oci_test/two/metadata.json"
expect_oci_rejection "derived OCI metadata substitution"
cp "$oci_test/one/metadata.json" "$oci_test/two/metadata.json"
printf 'sha256:%064d\n' 0 > "$oci_test/two/image.id"
expect_oci_rejection "loaded image substitution"
cp "$oci_test/one/image.id" "$oci_test/two/image.id"
/usr/bin/head -c 512 "$oci_test/one/image.tar" > "$oci_test/two/image.tar"
expect_oci_rejection "truncated archive"
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    --transform='s|manifest.json|../manifest.json|' -cf "$oci_test/two/image.tar" \
    -C "$oci_test/root" manifest.json
expect_oci_rejection "archive path escape"
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
printf 'changed layer\n' > "$oci_test/root/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/layer.tar"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" \
    manifest.json repositories "$digest.json" \
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/VERSION \
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/json \
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/layer.tar
expect_oci_rejection "layer diff-id substitution"
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 --test tests/preauth/src/main.rs -o "$test_dir/preauth-tests"
"$test_dir/preauth-tests" --test-threads=1
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 tests/preauth/src/records.rs -o "$test_dir/preauth-records"
"$test_dir/preauth-records"
printf '%s\n' \
    'target_execution=not-attempted' \
    'qemu_execution=not-attempted' \
    'emulator_execution=not-attempted' \
    'vm_execution=not-attempted' \
    'aws_calls=not-attempted'
