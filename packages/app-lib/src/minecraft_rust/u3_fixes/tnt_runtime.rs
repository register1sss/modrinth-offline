use super::{LAVA, PrimedTnt, TntFluidWorld, WATER};
use horizon_client::{Result, collisions::SceneCollisions};
use horizon_core::geometry::Vec3;

// PrimedTnt ground bounce must use the falling speed, not zeroed Y.
pub(super) fn finish_motion(
    wanted: Vec3,
    moved: Vec3,
    mut velocity: Vec3,
) -> (Vec3, bool) {
    let on_ground = wanted.y < 0.0 && moved.y != wanted.y;
    if moved.x != wanted.x {
        velocity.x = 0.0;
    }
    if moved.z != wanted.z {
        velocity.z = 0.0;
    }
    if moved.y != wanted.y && !on_ground {
        velocity.y = 0.0;
    }
    velocity =
        Vec3::new(velocity.x * 0.98, velocity.y * 0.98, velocity.z * 0.98);
    if on_ground {
        velocity =
            Vec3::new(velocity.x * 0.7, velocity.y * -0.5, velocity.z * 0.7);
    }
    (velocity, on_ground)
}
pub(super) fn tick(
    tnt: &mut PrimedTnt,
    world: &horizon_client::world::World,
    collision: &horizon_world::block_collision::BlockCollisionRegistry<'_>,
    assets: &horizon_client::assets::Assets,
    facts: &horizon_client::blocks::BlockFacts,
) -> Result<()> {
    tnt.previous = tnt.position;
    tnt.velocity.y -= 0.04;
    let mut scene = SceneCollisions {
        world,
        binding: collision,
        subject: None,
    };
    let wanted = tnt.velocity;
    let moved = scene
        .resolve(tnt.bounds(), wanted, 0.0, tnt.on_ground)
        .unwrap_or(Vec3::ZERO);
    tnt.position = Vec3::new(
        tnt.position.x + moved.x,
        tnt.position.y + moved.y,
        tnt.position.z + moved.z,
    );
    (tnt.velocity, tnt.on_ground) = finish_motion(wanted, moved, tnt.velocity);
    tnt.fuse -= 1;
    if tnt.fuse > 0 {
        // PrimedTnt updates immersion after moving and decrementing its fuse.
        let mut fluids =
            horizon_world::entity_fluid::EntityFluidInteraction::new([
                WATER, LAVA,
            ]);
        let mut fluid_world = TntFluidWorld {
            tnt,
            world,
            assets,
            facts,
        };
        fluids
            .update(&mut fluid_world, false, 4096)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        fluids.apply_current(&mut fluid_world, WATER, 0.014)?;
        fluids.apply_current(
            &mut fluid_world,
            LAVA,
            if world.fast_lava {
                0.007
            } else {
                0.0023333333333333335
            },
        )?;
    }
    Ok(())
}
