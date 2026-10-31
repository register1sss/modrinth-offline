# Bounded Horizon source audit

Pinned source: 3132240f21141518391d7d34d41a4f7b916c3c5a, game 26.3.

This is not a claim that every bug in Horizon or the launcher is fixed.

| Report | Source finding | Status |
| --- | --- | --- |
| Food continues in inventory | remote_handle_input returned before releasing use on menu screens | Fixed in 0.2.0, retained |
| Ridden chest-boat inventory cannot open | Inventory-key mount detection omitted chest boats on both client and server | Fixed in 0.2.0, retained |
| Straw bed breaks before night passes | Shift woke the sleeper, invoking destroy-on-leave | One confirmed cause fixed in 0.2.0; normal destroy-on-leave retained |
| Floating tridents | Local in_ground latch survived authoritative motion; removed support was not released | Earlier narrow fixes retained; not every floating-projectile cause ruled out |
| Goat horn silent | Item was recognized by remote item_has_use, but authoritative use_hand/throw_item had no instrument-use handler | Added registry/inline codec, use, sound, event, stats and cooldown in 0.3.0; gameplay/audio verification still needed |
| TNT erratic | Collision response zeroed vertical impact velocity before ground bounce | Missing bounce repaired in 0.3.0; server corrections/interpolation still require gameplay reproduction |
| Wind-charge delay | throw_wind_charge creates a projectile immediately; throw_item applies the stack cooldown and remote input waits for ticks | No arbitrary cooldown removal. Need timing trace to distinguish cooldown, tick stalls and packet/spawn delay |
| Spear animation | Separate attack/held first-person paths exist in first_person.rs and game/item_draw.rs | No confirmed safe correction yet; need exact view/hand/action reproduction |
| Dark patches after explosions | Explosion removal reaches level_set; World::set schedules light edits and dirty relighting | No justification to force global full relights. Need saved-world reproduction and light values before/after the blast |
| FPS drops / rendering errors | Not a single defect identifiable from the generic symptom | Need frame timing, renderer log, GPU/driver and reproducible scene |
| Platform crashes / ports | A successful Linux type check does not exercise Windows/macOS graphics or device drivers | Need actual crash report, OS, GPU/driver, action and whether in-frame/integrated/dedicated |

Unknown is not fixed, safe or compatible. No broad physics, rendering, world migration or dependency upgrades were applied without a specific finding.
