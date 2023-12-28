use bevy::{math::Vec3, render::primitives::Aabb};

pub fn intersect_aabb(lhs: (&Vec3, &Aabb), rhs: (&Vec3, &Aabb)) -> bool {
    let lhs_half_extents: Vec3 = lhs.1.half_extents.into();
    let rhs_half_extents: Vec3 = rhs.1.half_extents.into();

    let lhs = Aabb::from_min_max(*lhs.0 - lhs_half_extents, *lhs.0 + lhs_half_extents);
    let rhs = Aabb::from_min_max(*rhs.0 - rhs_half_extents, *rhs.0 + rhs_half_extents);

    lhs.min().cmple(rhs.max()).all() && lhs.max().cmpge(rhs.min()).all()
}
