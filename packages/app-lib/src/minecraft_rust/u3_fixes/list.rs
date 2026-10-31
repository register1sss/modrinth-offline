fn u3_fixes_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "u3_fixes".into(),
        title: "U3 Fixes 0.2.0".into(),
        description: "Исправление багов в майнкрафт на Rust".into(),
        keys: Vec::new(),
    });
}
