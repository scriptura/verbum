# LPC — YAML reference set

Reference examples aligned with **LPC-Paradigm-v1**.

These files are **reference samples**, not an exhaustive extraction of the LPC corpus. Their purpose is to provide stable exemplars of the conceptual boundaries established by the v1 document.

## Files

- `canonical_animations.yaml` — canonical `ExtractionProfile` and `CompositionProfile` declarations.
- `action_resolution.yaml` — declaration and AOT resolution of `AnimationAction`.
- `build_manifest.yaml` — conceptual reference shape for the build manifest / asset registry; v1 fixes its responsibility but does not fix a concrete schema.
- `lpc_base_64.yaml` — complete illustrative Small contract using the canonical LPC Character row layout.
- `lpc_weapon_128.yaml` — narrow physical example for the observed Medium block (`walk_128`).
- `lpc_weapon_192.yaml` — narrow physical example for the observed Large block (`thrust_192`).

## Important scope notes

The oversized contracts deliberately remain narrow because LPC-Paradigm-v1 does not provide a complete physical mapping for every oversized animation on a real asset. No missing rows or sequences have been invented merely to make the examples look exhaustive.

The `build_manifest.yaml` structure is likewise illustrative: the v1 document establishes that the build manifest / asset registry owns the global relation `DriverEquipmentId -> physical realizations / available buckets` and the `overlay` / `variant` classification, but does not establish a final YAML schema for those data.

## Validation performed

All six YAML files parse successfully. The canonical sequence indices were checked against their source `frame_count`, and resolution targets were checked against the declared canonical profiles.
