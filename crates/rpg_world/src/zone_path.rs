use bevy_math::{
    cubic_splines::{CubicCardinalSpline, CubicCurve, CubicGenerator},
    uvec2, UVec2, Vec2, Vec3,
};

use std::collections::VecDeque;

pub struct ZonePath(pub VecDeque<CubicCurve<Vec2>>);

impl ZonePath {
    pub fn generate() -> Self {
        let main_curve_points = [
            Vec2::new(-0.5, 0.),
            Vec2::new(0.0, 0.5),
            Vec2::new(0.25, 2.25),
            Vec2::new(4.25, 8.75),
            Vec2::new(8.75, 6.25),
            Vec2::new(12.25, 13.75),
            Vec2::new(16.5, 20.5),
            Vec2::new(26.5, 14.5),
            Vec2::new(22.5, 26.5),
            Vec2::new(32.0, 32.0),
            Vec2::new(32.0, 32.5),
        ];

        /*
        let main_curve = CubicCardinalSpline::new(0.5, main_curve_points).to_curve();
        let secondary_curve = CubicCardinalSpline::new(0.5, secondary_curve_points).to_curve();
        curves.push_back(main_curve.iter_positions(256).collect());
        curves.push_back(secondary_curve.iter_positions(256).collect());
        */
        let curve = CubicCardinalSpline::new(0.5, main_curve_points).to_curve();

        let mut curves: VecDeque<_> = VecDeque::new();
        curves.push_back(curve);

        Self(curves)
    }
}
