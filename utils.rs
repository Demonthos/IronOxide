use raylib::core::math::Vector2;
use specs::{Component, VecStorage};

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Position(pub Vector2);

pub fn to_tuple(v: Vector2) -> [f32; 2] {
    [v.x, v.y]
}

pub fn from_tuple(t: [f32; 2]) -> Vector2 {
    Vector2::new(t[0], t[1])
}
