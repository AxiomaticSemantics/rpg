use bevy::math::{Vec3, Vec3A};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub center: Vec3A,
    pub half_extents: Vec3A,
}

impl Aabb {
    pub fn from_min_max(minimum: Vec3, maximum: Vec3) -> Self {
        let minimum = Vec3A::from(minimum);
        let maximum = Vec3A::from(maximum);
        let center = 0.5 * (maximum + minimum);
        let half_extents = 0.5 * (maximum - minimum);
        Self {
            center,
            half_extents,
        }
    }

    #[inline(always)]
    pub fn min(&self) -> Vec3A {
        self.center - self.half_extents
    }

    #[inline(always)]
    pub fn max(&self) -> Vec3A {
        self.center + self.half_extents
    }
}

pub fn intersect_aabb(lhs: (&Vec3, &Aabb), rhs: (&Vec3, &Aabb)) -> bool {
    let lhs_half_extents: Vec3 = lhs.1.half_extents.into();
    let rhs_half_extents: Vec3 = rhs.1.half_extents.into();

    let lhs = Aabb::from_min_max(*lhs.0 - lhs_half_extents, *lhs.0 + lhs_half_extents);
    let rhs = Aabb::from_min_max(*rhs.0 - rhs_half_extents, *rhs.0 + rhs_half_extents);

    lhs.min().cmple(rhs.max()).all() && lhs.max().cmpge(rhs.min()).all()
}
