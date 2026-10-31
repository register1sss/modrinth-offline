# U3 Fixes 0.2.0

Pinned Horizon: 3132240f21141518391d7d34d41a4f7b916c3c5a, game 26.3.

- Stop active item use with one RELEASE_USE_ITEM packet when a remote client opens a menu. This also covers the default integrated-server client. Preserve the existing client-side Riptide release.
- Include ridden chest boats in the inventory-key path, on the client and server. Use the existing vehicle container and synchronization, without replacing slots belonging to an already-open chest. Do not fabricate remote inventories.
- Remove the non-vanilla Shift wake condition. Previously Shift could immediately wake the sleeper and trigger straw-bed destruction before 100 sleep ticks elapsed. Keep wake-on-death, missing-bed, sleeping percentage/time/weather rules and the existing Leave Bed path.
- Retain the earlier trident motion/support fixes and checksum-bound installation.

CPU regressions use original pinned method bodies with world/transport doubles. They reproduce six failures before the patch and pass ten checks afterward. Existing trident regressions remain seven checks. These are not GPU or full in-game tests.

Not claimed fixed here: wind-charge delay, TNT jitter, spear animation, missing horn use, rendering/lighting performance, platform crashes, or every possible cause of a floating trident. These require separate reproduction and checks.
