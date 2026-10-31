// Read the same instrument definitions that Horizon sends in RegistryData.
pub fn lookup(
    sync: &super::SyncData,
    id: &str,
) -> Option<horizon_world::nbt::Tag> {
    let (_, entries) = sync
        .registries
        .iter()
        .find(|(name, _)| name == "minecraft:instrument")?;
    let index = entries.names.iter().position(|name| name == id)?;
    entries.json.get(index).map(super::json_to_nbt)
}
