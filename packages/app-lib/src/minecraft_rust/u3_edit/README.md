# U3 Edit 0.1.0

Original small region editor by U3 for Horizon. Not affiliated with WorldEdit, no WorldEdit code.

- `/u3edit pos1` and `/u3edit pos2`: select the blocks at your feet. Optional `x y z` integer coordinates.
- `/u3edit info`: selection size.
- `/u3edit set minecraft:stone`, `/u3edit replace minecraft:dirt minecraft:stone`, `/u3edit walls minecraft:glass`: preview an edit.
- `/u3edit apply`: send the previewed vanilla command(s), once, within 60 seconds.
- `/u3edit cancel`, `/u3edit clear`, `/u3edit help`.

Maximum 32768 blocks in the entire selection, plain block IDs only. Vanilla/server permission checks apply. Cheats/operator rights are required for edits. No console escalation. The server reports success or failure; a sent command is not a success acknowledgement.

**No undo, clipboard, brushes, schematics, NBT or block-state syntax in this first version. Back up important worlds before editing.** Selection and pending edits are cleared on leaving the world or changing dimension. Native code is not sandboxed.

Source-only `.hnmod`, compiled by Horizon after explicit native-code consent. Hooks API 1; tested source revisions and mixin tool are pinned in `horizon-native.json`. Unit tests run directly with `rustc --test runtime.rs`. CI also applies the bundle to the pinned Horizon and checks the actual game crate. This does not replace a full in-game test on Windows.
