#!/bin/sh
set -eu
LC_ALL=C
LANG=C
export LC_ALL LANG
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)
fail() { printf 'controller-helper closure observer source check failed: %s\n' "$1" >&2; exit 1; }
observer=$root/tools/ci/observe-controller-helper-closure.sh
contract=$root/spec/alpha/lab/controller-helper-closure-observation-v0.fields
test_contract=$root/spec/alpha/lab/controller-helper-closure-observer-test-v0.fields
workflow=$root/.github/workflows/controller-helper-closure-observer.yml
wrapper=$root/tools/ci/run-controller-helper-closure-observer.sh
harness=$root/tools/ci/controller-helper-closure-observer-harness.sh
policy=$root/tools/ci/check-controller-helper-closure-observer-policy.sh
policy_test=$root/tools/ci/test-controller-helper-closure-observer-policy.sh
catalog=$root/tools/ci/fixtures/controller-helper-closure-observer/cases.v0
fixture=$root/tools/ci/fixtures/controller-helper-closure-observer/base-closure.v0
pins=$root/tools/ci/fixtures/controller-helper-closure-observer/tool-pins.v0
receipt=$root/tools/ci/fixtures/controller-helper-closure-observer/expected-observation.receipt.v0
sha() { /usr/bin/shasum -a 256 "$1" | /usr/bin/awk '{ print $1 }'; }
for file in "$observer" "$contract" "$test_contract" "$workflow" "$wrapper" "$harness" "$policy" "$policy_test" "$catalog" "$fixture" "$pins" "$receipt"; do
    [ -f "$file" ] && [ ! -L "$file" ] && [ -s "$file" ] || fail "required file missing: $file"
done
/bin/sh -n "$observer" "$wrapper" "$harness" "$policy" "$policy_test" || fail 'C2B shell syntax invalid'
for script in "$observer" "$wrapper" "$harness" "$policy" "$policy_test"; do
    [ -x "$script" ] || fail "required C2B script is not executable: $script"
done
[ "$(sha "$observer")" = 6fa10b187698077bfa96a119c376aeecb2ed4db92a25af3aad0f5add46e3b6cb ] || fail 'observer bytes escaped review'
[ "$(sha "$contract")" = 944229644f0805876403cd858d0d8c3c993d73bf00baef4c3ffcbbc7a2522836 ] || fail 'contract bytes escaped review'
[ "$(sha "$test_contract")" = d2f8837b52bfd2f5c77bc527b7106e981a3ed3bd7dd7114953a83316870045d4 ] || fail 'test_contract bytes escaped review'
[ "$(sha "$workflow")" = a277e658849523c0ceb0774128089fbb405a5c95e7311acceba5346f6bb98c19 ] || fail 'workflow bytes escaped review'
[ "$(sha "$wrapper")" = 70d7ce475553df5ce962ec45a6ac08bae776c70c04afd2ebeccfcba14a8a7ec0 ] || fail 'wrapper bytes escaped review'
[ "$(sha "$harness")" = 459d44d23d0e4036d233ab9bf1ab70b0b665352d2d7646f30d59ef67ab8b8677 ] || fail 'harness bytes escaped review'
[ "$(sha "$policy")" = 112e4f700e0427b429333b4d6fa13161b57d1a5963c66208bc0d69cbaa6df10f ] || fail 'policy bytes escaped review'
[ "$(sha "$policy_test")" = f896fdf82c3e5b5eec70cb23bd64b5edf0b878e5f40c8d9dae1328008be1cafb ] || fail 'policy_test bytes escaped review'
[ "$(sha "$catalog")" = 6b946f446d773389e924be0c8d35e88577b3e177f3da0d52af3f52cea398e289 ] || fail 'catalog bytes escaped review'
[ "$(sha "$fixture")" = bbef536b8820f962d3ce529de904421b9aa8cf808ad29953321f2b1cdafd0314 ] || fail 'fixture bytes escaped review'
[ "$(sha "$pins")" = c9cc0e778a0ee766d7e83286021c29aaff29ad55eea4431f921f4cc77972c3a9 ] || fail 'pins bytes escaped review'
[ "$(sha "$receipt")" = be60494cb57b03a8a0f48b36d8cd99ba7e0033a4b2ddb10222aa9b052935bbf4 ] || fail 'receipt bytes escaped review'

grep -Fqx 'execution_authority=C2-main-only-observer-harness-plus-one-candidate-observation-after-C2A-exact-main' "$test_contract" || fail 'C2 authority changed'
grep -Fqx 'authority_scope=O001-through-O021-isolated-harness+one-production-observer;no-compiler+helper+target+VM+readiness+trust-state-authority' "$test_contract" || fail 'C2 scope changed'
grep -Fqx '[ "${RAR_CONTROLLER_HELPER_CLOSURE_DISCOVERY-}" = 1 ] || fail '''explicit discovery mode is required'''' "$observer" || fail 'observer explicit mode missing'
grep -Fqx 'hardlinked=$(/usr/bin/find -P "$root" -type f -links +1 -printf x -quit) || fail '''cannot inspect closure hardlinks'''' "$observer" || fail 'observer no-hardlink predicate missing'
grep -Fqx '[ "$GITHUB_SHA" = "$RAR_TRUSTED_CONTROLLER_SHA" ] || fail '''controller is not exact main'''' "$observer" || fail 'controller identity check missing'
grep -Fqx '[ "$GITHUB_SHA" = "$RAR_EXPECTED_SOURCE_REVISION" ] || fail '''source is not exact main'''' "$observer" || fail 'source identity check missing'
grep -Fqx '/usr/bin/dash "$subject" || fail '''production observer failed'''' "$harness" || fail 'single production observer handoff missing'
/bin/sh "$policy" "$root" >/dev/null || fail 'C2B workflow policy invalid'
wired=$(/usr/bin/grep -Rl -F 'observe-controller-helper-closure.sh' "$root/.github/workflows" | /usr/bin/sort)
[ "$wired" = "$workflow" ] || fail 'observer workflow wiring is missing or duplicated'
grep -qx 'rust_toolchain_closure_manifest_relative=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure lock activated'
grep -qx 'rust_toolchain_closure_manifest_sha256=none' "$root/tools/toolchain/host-tools.x86_64-unknown-linux-gnu-ci.lock" || fail 'CI closure digest activated'
grep -qx 'state=blocked' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper inventory activated'
grep -qx 'compiler_closure_manifest_sha256=unavailable' "$root/tools/sprint-alpha/controller-helper-v0.env" || fail 'helper closure activated'
printf '%s\n' 'controller-helper closure observer C2B source is byte-bound, main-only, candidate-only, and non-activating'
