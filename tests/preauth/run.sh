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
mkdir -p out/r0/preauth/acquisition/host-tools
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 tools/toolchain/preauth-verify-oci.rs \
    -o out/r0/preauth/acquisition/host-tools/preauth-verify-oci
oci_test=out/r0/preauth/acquisition/derived-build/host-test
mkdir -p "$oci_test/root/blobs/sha256" \
    "$oci_test/one" "$oci_test/two"
printf 'minimal deterministic layer\n' > "$oci_test/root/layer.pending"
layer_digest=$(/usr/bin/sha256sum "$oci_test/root/layer.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/layer.pending" "$oci_test/root/blobs/sha256/$layer_digest"
printf '{"rootfs":{"type":"layers","diff_ids":["sha256:%s"]}}\n' "$layer_digest" > "$oci_test/root/config.pending"
digest=$(/usr/bin/sha256sum "$oci_test/root/config.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/config.pending" "$oci_test/root/blobs/sha256/$digest"
config_size=$(/usr/bin/wc -c < "$oci_test/root/blobs/sha256/$digest" | /usr/bin/tr -d ' ')
layer_size=$(/usr/bin/wc -c < "$oci_test/root/blobs/sha256/$layer_digest" | /usr/bin/tr -d ' ')
printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","digest":"sha256:%s","size":%s},"layers":[{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","size":%s}]}\n' \
    "$digest" "$config_size" "$layer_digest" "$layer_size" > "$oci_test/root/image-manifest.pending"
manifest_digest=$(/usr/bin/sha256sum "$oci_test/root/image-manifest.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/image-manifest.pending" "$oci_test/root/blobs/sha256/$manifest_digest"
manifest_size=$(/usr/bin/wc -c < "$oci_test/root/blobs/sha256/$manifest_digest" | /usr/bin/tr -d ' ')
printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:%s","size":%s}]}\n' \
    "$manifest_digest" "$manifest_size" > "$oci_test/root/index.json"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"]}]\n' \
    "$digest" "$head" "$layer_digest" > "$oci_test/root/manifest.json"
printf '{"imageLayoutVersion":"1.0.0"}\n' > "$oci_test/root/oci-layout"
write_repositories() {
    printf '{"rar-preauth":{"%s":"%s"}}\n' "$head" "$layer_digest" > "$oci_test/root/repositories"
}
write_repositories
build_two_archive() {
    build_two_members index.json manifest.json oci-layout repositories \
        "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
}
build_two_members() {
    printf '%s\n' "$@" | /usr/bin/tar --sort=name --mtime='@1784332800' \
        --owner=0 --group=0 --numeric-owner --format=gnu \
        -cf "$oci_test/two/image.tar" -C "$oci_test/root" -T -
}
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/one/image.tar" -C "$oci_test/root" \
    index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
printf '{"containerimage.config.digest":"sha256:%s","containerimage.digest":"sha256:%s"}\n' "$digest" "$manifest_digest" > "$oci_test/one/metadata.json"
cp "$oci_test/one/metadata.json" "$oci_test/two/metadata.json"
printf 'sha256:%s\n' "$digest" > "$oci_test/one/image.id"
cp "$oci_test/one/image.id" "$oci_test/two/image.id"
tools/toolchain/verify-preauth-oci.sh \
    "$oci_test/one/image.tar" "$oci_test/one/metadata.json" "$oci_test/one/image.id" \
    "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" >/dev/null
printf 'dangling graph member\n' > "$oci_test/root/orphan.pending"
orphan_digest=$(/usr/bin/sha256sum "$oci_test/root/orphan.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/orphan.pending" "$oci_test/root/blobs/sha256/$orphan_digest"
build_two_members index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest" \
    "blobs/sha256/$orphan_digest"
projection=$(out/r0/preauth/acquisition/host-tools/preauth-verify-oci \
    --member-list "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" - \
    2> "$oci_test/projection.log")
! printf '%s\n' "$projection" | /usr/bin/grep -Fqx "blobs/sha256/$orphan_digest"
/usr/bin/grep -Fqx 'oci_projection_omitted count=1' "$oci_test/projection.log"
mkdir -p "$oci_test/raw-root/blobs/sha256"
cp "$oci_test/root/index.json" "$oci_test/root/manifest.json" "$oci_test/root/oci-layout" \
    "$oci_test/root/repositories" "$oci_test/raw-root/"
cp "$oci_test/root/blobs/sha256/$digest" "$oci_test/root/blobs/sha256/$layer_digest" \
    "$oci_test/root/blobs/sha256/$manifest_digest" "$oci_test/root/blobs/sha256/$orphan_digest" \
    "$oci_test/raw-root/blobs/sha256/"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu --no-recursion \
    -cf "$oci_test/two/raw-with-directories.tar" -C "$oci_test/raw-root" \
    index.json manifest.json oci-layout repositories blobs blobs/sha256 \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" \
    "blobs/sha256/$manifest_digest" "blobs/sha256/$orphan_digest"
directory_projection=$(out/r0/preauth/acquisition/host-tools/preauth-verify-oci \
    --member-list "$oci_test/two/raw-with-directories.tar" \
    "$oci_test/two/metadata.json" "$oci_test/two/image.id" - 2> "$oci_test/directory-projection.log")
[ "$directory_projection" = "$projection" ]
/usr/bin/grep -Fqx 'oci_raw_directory path=blobs type=directory size=0 mode=0755 uid=0 gid=0' \
    "$oci_test/directory-projection.log"
/usr/bin/grep -Fqx 'oci_raw_directory path=blobs/sha256 type=directory size=0 mode=0755 uid=0 gid=0' \
    "$oci_test/directory-projection.log"
printf '%s\n' "$projection" | /usr/bin/tar --sort=name --mtime='@1784332800' \
    --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" -T -
cmp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
rm "$oci_test/root/blobs/sha256/$orphan_digest"
dangling_members=
for number in $(/usr/bin/seq 1 40); do
    printf 'bounded-diagnostic-%02d\n' "$number" > "$oci_test/root/orphan.pending"
    orphan_digest=$(/usr/bin/sha256sum "$oci_test/root/orphan.pending" | /usr/bin/cut -d ' ' -f 1)
    mv "$oci_test/root/orphan.pending" "$oci_test/root/blobs/sha256/$orphan_digest"
    dangling_members="$dangling_members blobs/sha256/$orphan_digest"
done
# shellcheck disable=SC2086
build_two_members index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest" \
    $dangling_members
for attempt in one two; do
    set +e
    tools/toolchain/verify-preauth-oci.sh \
        "$oci_test/one/image.tar" "$oci_test/one/metadata.json" "$oci_test/one/image.id" \
        "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" \
        > /dev/null 2> "$oci_test/diagnostic-$attempt.log"
    status=$?
    set -e
    [ "$status" -eq 73 ]
done
cmp "$oci_test/diagnostic-one.log" "$oci_test/diagnostic-two.log"
/usr/bin/grep -Fqx 'oci_unreachable_summary count=40 reported=32 cap=32' "$oci_test/diagnostic-one.log"
[ "$(/usr/bin/grep -c '^oci_unreachable path=' "$oci_test/diagnostic-one.log")" -eq 32 ]
for member in $dangling_members; do rm "$oci_test/root/$member"; done
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
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
rm "$oci_test/root/repositories"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" \
    index.json manifest.json oci-layout \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "missing repositories index"
write_repositories
build_two_members manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "missing OCI index"
build_two_members index.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "missing Docker manifest"
build_two_members index.json manifest.json repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "missing OCI layout"
build_two_members index.json manifest.json oci-layout repositories \
    "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "missing config"
build_two_members index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "missing layer"
build_two_members index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest"
expect_oci_rejection "missing OCI image manifest"
build_two_archive
printf '{' > "$oci_test/root/repositories"
build_two_archive
expect_oci_rejection "malformed repositories index"
printf '{"rar-preauth":{"%s":"%s"},"extra":{}}\n' "$head" "$layer_digest" > "$oci_test/root/repositories"
build_two_archive
expect_oci_rejection "extra repositories key"
printf '{"rar-preauth":{"%s":"%s"},"rar-preauth":{"%s":"%s"}}\n' \
    "$head" "$layer_digest" "$head" "$layer_digest" > "$oci_test/root/repositories"
build_two_archive
expect_oci_rejection "duplicate repositories key"
printf '{ "rar-preauth" : { "%s" : "%s" } }\n' "$head" "$layer_digest" > "$oci_test/root/repositories"
build_two_archive
expect_oci_rejection "noncanonical repositories encoding"
printf '{"rar-preauth":{"%s":"%s"}}\n' "$merge" "$layer_digest" > "$oci_test/root/repositories"
build_two_archive
expect_oci_rejection "wrong repositories tag"
printf '{"rar-preauth":{"%s":"%s"}}\n' "$head" "$digest" > "$oci_test/root/repositories"
build_two_archive
expect_oci_rejection "repositories config substitution"
write_repositories
printf 'unexpected\n' > "$oci_test/root/extra-member"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" \
    index.json manifest.json oci-layout repositories extra-member \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "extra archive member"
rm "$oci_test/root/extra-member"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" \
    index.json manifest.json oci-layout repositories repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "duplicate archive member"
/usr/bin/tar --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" \
    manifest.json index.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "noncanonical archive member order"
/usr/bin/head -c 513 /dev/zero > "$oci_test/root/repositories"
build_two_archive
expect_oci_rejection "oversized repositories member"
write_repositories
mkdir "$oci_test/root/repositories-directory"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    --transform='s|repositories-directory|repositories|' -cf "$oci_test/two/image.tar" \
    -C "$oci_test/root" repositories-directory
expect_oci_rejection "repositories member type"
rmdir "$oci_test/root/repositories-directory"
/usr/bin/tar --sort=name --mtime='@1784332800' --mode=0600 --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" \
    index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "repositories member mode"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=1 --group=1 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" \
    index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
expect_oci_rejection "repositories member ownership"
build_two_archive
printf '{"containerimage.config.digest":"sha256:%s","containerimage.digest":"sha256:%s"}\n' \
    "$digest" "$manifest_digest" > "$oci_test/two/metadata.json"
printf '{"rar-preauth":{"%s":"%s"}}\n' "$head" "$digest" > "$oci_test/root/repositories"
build_two_archive
expect_oci_rejection "substituted repositories bytes with unchanged metadata"
write_repositories
build_two_archive
cp "$oci_test/one/metadata.json" "$oci_test/two/metadata.json"
printf 'changed layer\n' > "$oci_test/root/blobs/sha256/$layer_digest"
/usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" \
    index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
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
