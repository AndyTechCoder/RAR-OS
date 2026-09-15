# Monitored network expiry wait
Cloud run34851403232 at source48db9c1dcdd237ae516961186a056dfcba8570cd and controller0c3b0134364acb8e0bdb2c7da942e3e3eb4fa3c2 passed native first/fresh/node and the complete malformed/drop/overflow campaign. It then refused the expiry scenario's 20-second delay because Vm.delay accepts at most five seconds. Both guests had reached Terminal; no guest panic was reported.

The scenario now performs four five-second monitored delays. Total requested wait remains20 seconds, the VM/pair deadline remains unchanged, and each delay services both guests through the existing companion/watchdog. Errors propagate immediately; there are no input-command retries. Pure mocks check all four calls and that a deadline error on the second call prevents further waiting.

This is a test-harness correction only. The target expiry grant, clocks, capabilities, network topology and acceptance oracle are unchanged. Complete expiry/peer-death/integrated journeys remain unproven until an actual cloud run succeeds.
