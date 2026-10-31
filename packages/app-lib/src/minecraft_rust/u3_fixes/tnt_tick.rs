fn u3_tnt_tick(
    tnt: &mut crate::game::tnt::PrimedTnt,
    world: &horizon_client::world::World,
    collision: &horizon_world::block_collision::BlockCollisionRegistry<'_>,
    assets: &horizon_client::assets::Assets,
    facts: &horizon_client::blocks::BlockFacts,
) -> core::ops::ControlFlow<Result<()>> {
    core::ops::ControlFlow::Break(crate::game::tnt::u3_tnt::tick(
        tnt, world, collision, assets, facts,
    ))
}
