#!/bin/sh
set -eu

assertion_id=bootstrap
assertion_category=setup
assertion_expected=script-completes
assertion_actual=command-exit
report_assertion_failure() {
    status=$?
    trap - 0
    if [ "$status" -ne 0 ]; then
        printf 'ASSERTION_FAIL id=%s category=%s expected=%s actual=%s status=%s\n' \
            "$assertion_id" "$assertion_category" "$assertion_expected" "$assertion_actual" "$status" >&2
    fi
    exit "$status"
}
trap report_assertion_failure 0
mark_assertion() {
    assertion_id=$1
    assertion_category=$2
    assertion_expected=$3
    assertion_actual=${4:-pending}
}
assert_equal() {
    mark_assertion "$1" "$2" "$3" "$4"
    [ "$3" = "$4" ]
}
assert_status() {
    mark_assertion "$1" rejection "$2" "$3"
    [ "$2" -eq "$3" ]
}
assert_file_line() {
    mark_assertion "$1" fixture-log "$2" bounded-file-line
    /usr/bin/grep -Fqx "$2" "$3"
}
assert_files_equal() {
    mark_assertion "$1" deterministic-bytes equal "$2,$3"
    cmp "$2" "$3"
}

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
cd "$root"

tests/preauth/output-ownership.sh

head=0123456789abcdef0123456789abcdef01234567
merge=89abcdef0123456789abcdef0123456789abcdef
mark_assertion head.push positive-binding "source_revision=$head"
actual=$(tools/toolchain/bind-preauth-head.sh push "$head" - "$head")
assert_equal head.push positive-binding "source_revision=$head" "$actual"
mark_assertion head.pull_request positive-binding "source_revision=$head"
actual=$(tools/toolchain/bind-preauth-head.sh pull_request "$merge" "$head" "$head")
assert_equal head.pull_request positive-binding "source_revision=$head" "$actual"
rejected_index=0
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
    rejected_index=$((rejected_index + 1))
    assert_status "head.reject.$rejected_index" 73 "$status"
done

mark_assertion fixture.setup setup success command-exit
test_dir=out/r0/preauth/host-tests
mkdir -p "$test_dir"
mkdir -p out/r0/preauth/acquisition/host-tools
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 tools/toolchain/preauth-verify-oci.rs \
    -o out/r0/preauth/acquisition/host-tools/preauth-verify-oci
oci_test=out/r0/preauth/acquisition/derived-build/host-test
json_builder=out/r0/preauth/acquisition/host-tools/preauth-verify-oci
mkdir -p "$oci_test/root/blobs/sha256" \
    "$oci_test/one" "$oci_test/two"
printf 'minimal deterministic layer\n' > "$oci_test/root/layer.pending"
layer_digest=$(/usr/bin/sha256sum "$oci_test/root/layer.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/layer.pending" "$oci_test/root/blobs/sha256/$layer_digest"
printf '{"rootfs":{"type":"layers","diff_ids":["sha256:%s"]}}\n' "$layer_digest" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/config.pending"
digest=$(/usr/bin/sha256sum "$oci_test/root/config.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/config.pending" "$oci_test/root/blobs/sha256/$digest"
config_size=$(/usr/bin/wc -c < "$oci_test/root/blobs/sha256/$digest" | /usr/bin/tr -d ' ')
layer_size=$(/usr/bin/wc -c < "$oci_test/root/blobs/sha256/$layer_digest" | /usr/bin/tr -d ' ')
printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"mediaType":"application/vnd.oci.image.config.v1+json","digest":"sha256:%s","size":%s},"layers":[{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","size":%s}]}\n' \
    "$digest" "$config_size" "$layer_digest" "$layer_size" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/image-manifest.pending"
manifest_digest=$(/usr/bin/sha256sum "$oci_test/root/image-manifest.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/image-manifest.pending" "$oci_test/root/blobs/sha256/$manifest_digest"
manifest_size=$(/usr/bin/wc -c < "$oci_test/root/blobs/sha256/$manifest_digest" | /usr/bin/tr -d ' ')
write_index() {
    printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:%s","size":%s,"annotations":{"io.containerd.image.name":"docker.io/library/rar-preauth:%s","org.opencontainers.image.ref.name":"%s"}}]}' \
        "$1" "$2" "$head" "$head" | "$json_builder" --canonicalize-json bare > "$oci_test/root/index.json"
}
write_index "$manifest_digest" "$manifest_size"
write_manifest() {
    printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%s":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","size":%s}}}]\n' \
        "$digest" "$head" "$layer_digest" "$layer_digest" "$layer_digest" "$layer_size" | \
        "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
}
write_manifest
printf '{"imageLayoutVersion":"1.0.0"}\n' | "$json_builder" --canonicalize-json line > "$oci_test/root/oci-layout"
write_repositories() {
    printf '{"rar-preauth":{"%s":"%s"}}\n' "$head" "$layer_digest" | \
        "$json_builder" --canonicalize-json line > "$oci_test/root/repositories"
}
write_repositories
build_two_archive() {
    build_two_members index.json manifest.json oci-layout repositories \
        "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest"
}
build_two_members() {
    printf '%s\n' "$@" | LC_ALL=C /usr/bin/sort | \
        /usr/bin/tar --sort=name --mtime='@1784332800' \
        --owner=0 --group=0 --numeric-owner --format=gnu \
        -cf "$oci_test/two/image.tar" -C "$oci_test/root" -T -
}
printf '%s\n' \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest" \
    index.json manifest.json oci-layout repositories | LC_ALL=C /usr/bin/sort | \
    /usr/bin/tar --sort=name --mtime='@1784332800' --owner=0 --group=0 \
        --numeric-owner --format=gnu -cf "$oci_test/one/image.tar" -C "$oci_test/root" -T -
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
printf '{"containerimage.config.digest":"sha256:%s","containerimage.digest":"sha256:%s"}\n' \
    "$digest" "$digest" | "$json_builder" --canonicalize-json line > "$oci_test/one/metadata.json"
cp "$oci_test/one/metadata.json" "$oci_test/two/metadata.json"
printf 'sha256:%s\n' "$digest" > "$oci_test/one/image.id"
cp "$oci_test/one/image.id" "$oci_test/two/image.id"
mark_assertion fixture.positive-verifier positive-verification success command-exit
tools/toolchain/verify-preauth-oci.sh \
    "$oci_test/one/image.tar" "$oci_test/one/metadata.json" "$oci_test/one/image.id" \
    "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" >/dev/null
printf 'dangling graph member\n' > "$oci_test/root/orphan.pending"
orphan_digest=$(/usr/bin/sha256sum "$oci_test/root/orphan.pending" | /usr/bin/cut -d ' ' -f 1)
mv "$oci_test/root/orphan.pending" "$oci_test/root/blobs/sha256/$orphan_digest"
build_two_members index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$manifest_digest" \
    "blobs/sha256/$orphan_digest"
mark_assertion fixture.projection verifier-projection success command-exit
projection=$(out/r0/preauth/acquisition/host-tools/preauth-verify-oci \
    --member-list "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" - \
    2> "$oci_test/projection.log")
mark_assertion fixture.projection.orphan-excluded graph-authority absent bounded-member-list
if printf '%s\n' "$projection" | /usr/bin/grep -Fqx "blobs/sha256/$orphan_digest"; then false; fi
assert_file_line fixture.projection.omitted-count 'oci_projection_omitted count=1' "$oci_test/projection.log"
mark_assertion fixture.raw-directory-setup setup success command-exit
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
mark_assertion fixture.directory-projection verifier-projection success command-exit
directory_projection=$(out/r0/preauth/acquisition/host-tools/preauth-verify-oci \
    --member-list "$oci_test/two/raw-with-directories.tar" \
    "$oci_test/two/metadata.json" "$oci_test/two/image.id" - 2> "$oci_test/directory-projection.log")
mark_assertion fixture.directory-projection.members deterministic-graph equal member-list-mismatch
[ "$directory_projection" = "$projection" ]
assert_file_line fixture.directory-projection.blobs \
    'oci_raw_directory path=blobs type=directory size=0 mode=0755 uid=0 gid=0' \
    "$oci_test/directory-projection.log"
assert_file_line fixture.directory-projection.sha256 \
    'oci_raw_directory path=blobs/sha256 type=directory size=0 mode=0755 uid=0 gid=0' \
    "$oci_test/directory-projection.log"
mark_assertion fixture.canonical-projection setup success command-exit
printf '%s\n' "$projection" | /usr/bin/tar --sort=name --mtime='@1784332800' \
    --owner=0 --group=0 --numeric-owner --format=gnu \
    -cf "$oci_test/two/image.tar" -C "$oci_test/root" -T -
assert_files_equal fixture.canonical-projection.bytes "$oci_test/one/image.tar" "$oci_test/two/image.tar"
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
    assert_status "fixture.unreachable.reject.$attempt" 73 "$status"
done
assert_files_equal fixture.unreachable.diagnostic-stable "$oci_test/diagnostic-one.log" "$oci_test/diagnostic-two.log"
assert_file_line fixture.unreachable.summary 'oci_unreachable_summary count=40 reported=32 cap=32' "$oci_test/diagnostic-one.log"
actual=$(/usr/bin/grep -c '^oci_unreachable path=' "$oci_test/diagnostic-one.log")
assert_equal fixture.unreachable.cap bounded-diagnostics 32 "$actual"
for member in $dangling_members; do rm "$oci_test/root/$member"; done
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
expect_oci_rejection() {
    description=$1
    negative_index=$((negative_index + 1))
    mark_assertion "fixture.negative.$negative_index" negative-rejection "$description" pending
    set +e
    tools/toolchain/verify-preauth-oci.sh \
        "$oci_test/one/image.tar" "$oci_test/one/metadata.json" "$oci_test/one/image.id" \
        "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" >/dev/null 2>&1
    status=$?
    set -e
    assertion_actual=$status
    [ "$status" -eq 73 ]
}
negative_index=0
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"]}]\n' \
    "$digest" "$head" "$layer_digest" | "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "missing Docker LayerSources"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%s":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","size":%s,"unknown":true}}}]\n' \
    "$digest" "$head" "$layer_digest" "$layer_digest" "$layer_digest" "$layer_size" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "unknown Docker LayerSources descriptor key"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%064d":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","size":%s}}}]\n' \
    "$digest" "$head" "$layer_digest" 0 "$layer_digest" "$layer_size" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "Docker LayerSources diff-id substitution"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%s":{"mediaType":"application/vnd.oci.image.layer.v1.tar+gzip","digest":"sha256:%s","size":%s}}}]\n' \
    "$digest" "$head" "$layer_digest" "$layer_digest" "$layer_digest" "$layer_size" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "Docker LayerSources media-type substitution"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%s":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","size":"%s"}}}]\n' \
    "$digest" "$head" "$layer_digest" "$layer_digest" "$layer_digest" "$layer_size" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "Docker LayerSources size type confusion"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%s":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%064d","size":%s}}}]\n' \
    "$digest" "$head" "$layer_digest" "$layer_digest" 0 "$layer_size" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "Docker LayerSources digest cross-link substitution"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%s":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","size":%s}}}]\n' \
    "$digest" "$head" "$layer_digest" "$layer_digest" "$layer_digest" "$((layer_size + 1))" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "Docker LayerSources exact payload size mismatch"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%064d":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%064d","size":1},"sha256:%s":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","size":%s}}}]\n' \
    "$digest" "$head" "$layer_digest" 0 0 "$layer_digest" "$layer_digest" "$layer_size" | \
    "$json_builder" --canonicalize-json line > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "extra Docker LayerSources row"
printf '[{"Config":"blobs/sha256/%s","RepoTags":["rar-preauth:%s"],"Layers":["blobs/sha256/%s"],"LayerSources":{"sha256:%s":{"mediaType":"application/vnd.oci.image.layer.v1.tar","digest":"sha256:%s","digest":"sha256:%s","size":%s}}}]\n' \
    "$digest" "$head" "$layer_digest" "$layer_digest" "$layer_digest" "$layer_digest" "$layer_size" \
    > "$oci_test/root/manifest.json"
build_two_archive
expect_oci_rejection "duplicate Docker LayerSources descriptor key"
write_manifest
build_two_archive
printf 'different-archive\n' > "$oci_test/two/image.tar"
expect_oci_rejection "derived OCI byte mismatch"
cp "$oci_test/one/image.tar" "$oci_test/two/image.tar"
printf '{"containerimage.digest": "sha256:%064d"}\n' 0 > "$oci_test/two/metadata.json"
expect_oci_rejection "derived OCI metadata substitution"
cp "$oci_test/one/metadata.json" "$oci_test/two/metadata.json"
printf '{"containerimage.config.digest":"sha256:%s","containerimage.digest":"sha256:%s"}\n' \
    "$digest" "$manifest_digest" > "$oci_test/two/metadata.json"
expect_oci_rejection "Buildx config identity relabeled as OCI manifest"
printf '{"containerimage.config.digest":"sha256:%s","containerimage.descriptor":{"digest":"sha256:%s","mediaType":"application/vnd.oci.image.manifest.v1+json","platform":{"architecture":"amd64","os":"linux"}},"containerimage.digest":"sha256:%s"}\n' \
    "$digest" "$digest" "$digest" > "$oci_test/two/metadata.json"
expect_oci_rejection "metadata-only descriptor spoof"
cp "$oci_test/one/metadata.json" "$oci_test/two/metadata.json"
printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:%s","size":%s,"platform":{"architecture":"arm64","os":"linux"}}]}\n' \
    "$manifest_digest" "$manifest_size" > "$oci_test/root/index.json"
build_two_archive
for attempt in one two; do
    set +e
    out/r0/preauth/acquisition/host-tools/preauth-verify-oci --member-list \
        "$oci_test/two/image.tar" "$oci_test/two/metadata.json" "$oci_test/two/image.id" - \
        > /dev/null 2> "$oci_test/index-diagnostic-$attempt.log"
    status=$?
    set -e
    assert_status "fixture.index-diagnostic.reject.$attempt" 73 "$status"
done
assert_files_equal fixture.index-diagnostic.stable "$oci_test/index-diagnostic-one.log" "$oci_test/index-diagnostic-two.log"
mark_assertion fixture.index-diagnostic.descriptor fixture-log present bounded-file-line
/usr/bin/grep -F 'oci_index_descriptor count=1 order=archive-index-order' \
    "$oci_test/index-diagnostic-one.log" >/dev/null
mark_assertion fixture.index-diagnostic.architecture fixture-log present bounded-file-line
/usr/bin/grep -F 'actual_architecture=["arm64"] expected_architecture=[]' \
    "$oci_test/index-diagnostic-one.log" >/dev/null
mark_assertion fixture.index-diagnostic.annotations fixture-log present bounded-file-line
/usr/bin/grep -F 'oci_index_annotations count=0 keys_and_value_hashes=[]' \
    "$oci_test/index-diagnostic-one.log" >/dev/null
expect_oci_rejection "Buildx platform substitution"
write_index "$manifest_digest" "$manifest_size"
printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:%s","size":%s,"annotations":{"org.opencontainers.image.ref.name":"%s"}}]}\n' \
    "$manifest_digest" "$manifest_size" "$head" > "$oci_test/root/index.json"
build_two_archive
expect_oci_rejection "missing containerd image-name annotation"
printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:%s","size":%s,"annotations":{"io.containerd.image.name":"docker.io/library/rar-preauth:%s","org.opencontainers.image.ref.name":"%s","unexpected":"value"}}]}\n' \
    "$manifest_digest" "$manifest_size" "$head" "$head" > "$oci_test/root/index.json"
build_two_archive
expect_oci_rejection "unknown OCI index annotation"
printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:%s","size":%s,"annotations":{"io.containerd.image.name":"docker.io/library/rar-preauth:%s","org.opencontainers.image.ref.name":"%s"}}]}\n' \
    "$manifest_digest" "$manifest_size" "$head" "$merge" > "$oci_test/root/index.json"
build_two_archive
expect_oci_rejection "substituted OCI index ref annotation"
printf '{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:%s","size":%s,"annotations":{"org.opencontainers.image.ref.name":"%s","io.containerd.image.name":"docker.io/library/rar-preauth:%s"}}]}\n' \
    "$manifest_digest" "$manifest_size" "$head" "$head" > "$oci_test/root/index.json"
build_two_archive
expect_oci_rejection "noncanonical OCI index annotation order"
write_index "$manifest_digest" "$manifest_size"
cp "$oci_test/root/blobs/sha256/$manifest_digest" "$oci_test/root/image-manifest.bad"
/usr/bin/sed -i 's|application/vnd.oci.image.config.v1+json|application/vnd.oci.image.config.v1+json-spoof|' \
    "$oci_test/root/image-manifest.bad"
bad_manifest_digest=$(/usr/bin/sha256sum "$oci_test/root/image-manifest.bad" | /usr/bin/cut -d ' ' -f 1)
bad_manifest_size=$(/usr/bin/wc -c < "$oci_test/root/image-manifest.bad" | /usr/bin/tr -d ' ')
mv "$oci_test/root/image-manifest.bad" "$oci_test/root/blobs/sha256/$bad_manifest_digest"
write_index "$bad_manifest_digest" "$bad_manifest_size"
build_two_members index.json manifest.json oci-layout repositories \
    "blobs/sha256/$digest" "blobs/sha256/$layer_digest" "blobs/sha256/$bad_manifest_digest"
expect_oci_rejection "transformed OCI media-type substitution"
rm "$oci_test/root/blobs/sha256/$bad_manifest_digest"
write_index "$manifest_digest" "$manifest_size"
build_two_archive
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
    "$digest" "$digest" > "$oci_test/two/metadata.json"
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
mark_assertion host-tests.compile rust-host-tests success command-exit
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 --test tests/preauth/src/main.rs -o "$test_dir/preauth-tests"
mark_assertion host-tests.run rust-host-tests pass command-exit
"$test_dir/preauth-tests" --test-threads=1
mark_assertion records.compile record-host-tests success command-exit
/usr/local/rustup/toolchains/1.95.0-x86_64-unknown-linux-gnu/bin/rustc \
    --edition=2024 tests/preauth/src/records.rs -o "$test_dir/preauth-records"
mark_assertion records.run record-host-tests pass command-exit
"$test_dir/preauth-records"
mark_assertion safety.evidence non-execution-evidence emitted command-exit
printf '%s\n' \
    'target_execution=not-attempted' \
    'qemu_execution=not-attempted' \
    'emulator_execution=not-attempted' \
    'vm_execution=not-attempted' \
    'aws_calls=not-attempted'
