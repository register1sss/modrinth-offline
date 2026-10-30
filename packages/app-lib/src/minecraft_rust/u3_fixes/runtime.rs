// U3 Fixes 0.1.0. Exact-revision client-state repair, not a physics replacement.
use super::{Game, trident::ThrownTrident};
use horizon_client::collisions::SceneCollisions;
use horizon_core::geometry::{Aabb, Vec3};

// A local ray hit zeros velocity and latches in_ground. A later server motion
// packet previously updated only velocity, leaving remote_tick stuck forever.
// Clear the prediction latch, not Loyalty, ownership, pickup, or damage state.
pub(super) fn authoritative_motion(t: &mut ThrownTrident, v: [f64; 3]) {
    if v.iter().all(|x| x.is_finite()) && v.iter().any(|x| *x != 0.0) {
        t.in_ground = false;
    }
}

// AbstractArrow.shouldFall uses a 0.06 inflated point, not the projectile's
// visual bounding box. Fail closed on missing chunks / shape-query errors.
pub(super) fn release_unsupported(game: &mut Game<'_>) {
    let mut scene = SceneCollisions {
        world: &game.world,
        binding: game.collision,
        subject: None,
    };
    for t in &mut game.tridents {
        if !t.in_ground || t.no_physics {
            continue;
        }
        let p = t.position;
        if ![p.x, p.y, p.z].iter().all(|x| x.is_finite()) {
            continue;
        }
        let bounds = Aabb::new(p, p).inflate(Vec3::new(0.06, 0.06, 0.06));
        let lo = bounds.min_position();
        let hi = bounds.max_position();
        let loaded = [lo.x, hi.x].into_iter().all(|x| {
            [lo.z, hi.z]
                .into_iter()
                .all(|z| game.world.loaded(x.floor() as i32, z.floor() as i32))
        });
        if loaded {
            support_result(t, scene.unobstructed(bounds).ok());
        }
    }
}

// None means an unavailable collision query, never an empty world.
fn support_result(t: &mut ThrownTrident, clear: Option<bool>) {
    if t.in_ground && !t.no_physics && clear == Some(true) {
        t.in_ground = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_client::{mobs::Who, projectiles::ProjectileWorld};
    struct Empty;
    impl ProjectileWorld for Empty {
        fn clip(&self, _: Vec3, _: Vec3) -> Option<(Vec3, [i32; 3], [i32; 3])> {
            None
        }
        fn in_water(&self, _: Aabb) -> bool {
            false
        }
        fn targets(&self) -> &[(Who, Aabb)] {
            &[]
        }
    }
    fn stuck() -> ThrownTrident {
        let mut t =
            ThrownTrident::remote(42, Vec3::new(3.0, 4.0, 5.0), 0.0, 0.0);
        t.in_ground = true;
        t
    }
    #[test]
    fn server_motion_resumes_real_remote_tick() {
        let mut t = stuck();
        authoritative_motion(&mut t, [0.5, 0.0, 0.0]);
        // The existing remote_motion body performs this assignment.
        t.velocity = Vec3::new(0.5, 0.0, 0.0);
        t.remote_tick(&Empty, None);
        assert_eq!(t.position.x, 3.5);
        assert!(!t.in_ground);
        assert!(t.velocity.y < 0.0);
    }
    #[test]
    fn zero_packet_keeps_legitimate_block_hit() {
        let mut t = stuck();
        authoritative_motion(&mut t, [0.0; 3]);
        t.remote_tick(&Empty, None);
        assert!(t.in_ground);
        assert_eq!(t.position.x, 3.0);
    }
    #[test]
    fn invalid_motion_does_not_unstick() {
        for v in [[f64::NAN, 1.0, 0.0], [f64::INFINITY, 0.0, 0.0]] {
            let mut t = stuck();
            authoritative_motion(&mut t, v);
            assert!(t.in_ground);
        }
    }
    #[test]
    fn removed_support_resumes_gravity() {
        let mut t = stuck();
        support_result(&mut t, Some(true));
        t.remote_tick(&Empty, None);
        t.remote_tick(&Empty, None);
        assert!(!t.in_ground);
        assert!(t.position.y < 4.0);
    }
    #[test]
    fn existing_support_and_query_errors_keep_state() {
        for clear in [Some(false), None] {
            let mut t = stuck();
            support_result(&mut t, clear);
            assert!(t.in_ground);
        }
    }
    #[test]
    fn support_probe_does_not_change_returning_tridents() {
        let mut t = stuck();
        t.no_physics = true;
        support_result(&mut t, Some(true));
        assert!(t.in_ground);
        assert!(t.no_physics);
    }
    #[test]
    fn ownership_and_loyalty_are_preserved() {
        let mut t = stuck();
        t.owner = 99;
        t.loyalty = 3;
        authoritative_motion(&mut t, [0.0, -0.2, 0.0]);
        assert_eq!(t.owner, 99);
        assert_eq!(t.loyalty, 3);
        assert_eq!(&*t.stack.item, "minecraft:trident");
    }
}
