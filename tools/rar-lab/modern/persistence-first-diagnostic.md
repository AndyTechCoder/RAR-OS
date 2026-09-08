# First Modern cloud diagnostic: pre-start ownership refusal

Run34271876157, job102215185348, trusted controller f441533bcb6763491ec00c9d280b0910826f5d58,
source6fefabe4bac1fc967ba0c0b73743fe9c844964db stopped in owned_identity during
the first compiler container's pre-start inspection. No target build or VM start
occurred in that run. Artifact10074144358 retains the failure manifest.

The original error groups ID, name, image and labels, so the exact mismatched
field is not proven. Inherited pinned-image labels are a plausible compatibility
cause, not a confirmed diagnosis. The corrected controller carries the exact
image-inspected label map into container validation, adds only its reserved
invocation label, and requires exact equality on both created and exited objects.
An image that already declares the reserved invocation label is refused.
Identity, image, confinement and ID-only cleanup rules remain unchanged.

The error now reports only match booleans for ID/name/image/labels, never arbitrary
environment or label values. Pure tests cover inherited metadata, unexpected or
changed labels, wrong ownership, reserved-key collision and full inherited-label
lifecycle success. A subsequent reviewed cloud run must confirm compatibility;
no retry, boot, persistence or milestone success is claimed by this correction.
