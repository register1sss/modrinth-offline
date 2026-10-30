fn u3_fixes_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod{id:"u3_fixes".into(),title:"U3 Fixes 0.1.0".into(),description:"Trident client state: resume authoritative motion and release after its supporting collision shape disappears. Pinned to Horizon 3132240. Does not fix all video issues.".into(),keys:Vec::new()});
}
