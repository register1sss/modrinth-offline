# U3 Fixes 0.3.0

- Goat horn: resolve the default, named or inline instrument; start timed use, play the authoritative sound, record use statistics and a sculk game event, apply the instrument-duration cooldown and synchronize it for guests. The remote client predicts use without duplicating server audio. Registry definitions, including server-provided definitions, are preferred over hard-coded horn indexes. Invalid non-finite duration/range values are rejected.
- TNT: retain falling vertical speed for the ground bounce instead of zeroing it before multiplying by -0.5. Keep the pinned collision shapes, gravity, drag, fluid-current handling, fuse and server-only explosion path. This repairs the missing bounce, not every possible source of network jitter.
- Retain all 0.2.0 and trident fixes. No save format, game version, native-code trust or checksum checks are disabled.

CPU checks: 4 TNT response tests (the ground bounce fails on the original code), 8 instrument-codec tests, 10 prior gameplay regressions and 7 trident regressions. Mixin transformation and full Horizon type checking are separate CI checks. These do not substitute for actual gameplay/audio/GPU tests.

# U3 Fixes 0.2.0

Pinned Horizon: 3132240f21141518391d7d34d41a4f7b916c3c5a, game 26.3.

- Stop active item use with one RELEASE_USE_ITEM packet when a remote client opens a menu. This also covers the default integrated-server client. Preserve the existing client-side Riptide release.
- Include ridden chest boats in the inventory-key path, on the client and server. Use the existing vehicle container and synchronization, without replacing slots belonging to an already-open chest. Do not fabricate remote inventories.
- Remove the non-vanilla Shift wake condition. Previously Shift could immediately wake the sleeper and trigger straw-bed destruction before 100 sleep ticks elapsed. Keep wake-on-death, missing-bed, sleeping percentage/time/weather rules and the existing Leave Bed path.
- Retain the earlier trident motion/support fixes and checksum-bound installation.

CPU regressions use original pinned method bodies with world/transport doubles. They reproduce six failures before the patch and pass ten checks afterward. Existing trident regressions remain seven checks. These are not GPU or full in-game tests.

Not claimed fixed here: wind-charge delay, TNT jitter, spear animation, missing horn use, rendering/lighting performance, platform crashes, or every possible cause of a floating trident. These require separate reproduction and checks.
