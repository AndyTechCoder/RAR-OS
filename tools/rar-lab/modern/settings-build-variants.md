# Standalone Settings build inventory


The compile-only Modern build recipe additionally builds three standalone UEFI
Settings executables from the exact source checkout:
factory with rar_settings_only; update additionally with rar_settings_v2;
failed-health with both plus rar_settings_fail_health. The fixed transfer order
is kernel, supervisor service, factory, update, failed-health. Missing, extra or
reordered entries are rejected; the existing 6MiB transfer bound is unchanged.

The controller applies service PE/W^X/128KiB mapped-image restrictions to all
three Settings files, records exact SHA256 and fixed recipe cfgs, rejects equal
variant payloads, and compares every artifact across two independent builds.
These cfg claims are bound to the trusted controller/build recipe, not supplied
by a package caller. Byte differences are necessary but not proof of health/UI
behavior. The bad-signature and bad-ABI packages intentionally reuse the update
executable while changing the corresponding package fields; only failed-health
requires distinct failing application code.

This compile-only extension does not select the signed kernel/supervisor cfg or
provision System media. Trusted-controller integration, package signing of these
exact returned bytes, immutable-bank embedding/inspection, and certified-VM
behavior remain gates. Existing published artifact records retain their original
controller/source identities and are not reinterpreted as this new inventory.
