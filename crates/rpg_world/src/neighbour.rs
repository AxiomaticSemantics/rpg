use glam::UVec2;

pub trait Neighbour {
    fn top(&self) -> Self;
    fn bottom(&self) -> Self;
    fn right(&self) -> Self;
    fn left(&self) -> Self;
}

impl Neighbour for UVec2 {
    fn top(&self) -> Self {
        let mut top = *self;
        top.y = top.y.saturating_sub(1);
        top
    }

    fn bottom(&self) -> Self {
        let mut bottom = *self;
        bottom.y += 1;
        bottom
    }

    fn left(&self) -> Self {
        let mut left = *self;
        left.x = left.x.saturating_sub(1);
        left
    }

    fn right(&self) -> Self {
        let mut right = *self;
        right.x += 1;
        right
    }
}
