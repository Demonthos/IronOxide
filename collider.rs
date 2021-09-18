use raylib::math::Vector2;
use specs::{Component, VecStorage};

/// handles narrow phase collisions, and generating aabbs.
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Collider {
    pub shape: Shape,
    pub physics_collider: bool,
}

impl Collider {
    pub fn get_collision(
        &self,
        pos: &Vector2,
        other_pos: &Vector2,
        other: &Collider,
    ) -> Option<Vector2> {
        self.shape.get_collision(pos, other_pos, &other.shape)
    }

    pub fn get_collision_bounds(&self, pos: &Vector2, bounds: [f32; 4]) -> Option<Vector2> {
        self.shape.get_collision_bounds(pos, bounds)
    }

    pub fn get_bounding_box(&self, pos: &Vector2) -> [Vector2; 2] {
        self.shape.get_bounding_box(pos)
    }
}

#[derive(Debug, Clone)]
pub enum Shape {
    CircleCollider { radius: f32 },
    RectangeCollider { size: Vector2 },
}

impl Shape {
    fn get_collision(&self, pos: &Vector2, other_pos: &Vector2, other: &Shape) -> Option<Vector2> {
        match self {
            Shape::CircleCollider { radius } => match other {
                Shape::CircleCollider {
                    radius: other_radius,
                } => {
                    let mut collision_vec = (*other_pos + Vector2::one() * (*other_radius))
                        - (*pos + Vector2::one() * (*radius));
                    let mut dist = collision_vec.length_sqr();
                    let sum_r = radius + other_radius;
                    if dist <= sum_r * sum_r {
                        dist = dist.sqrt();
                        collision_vec.normalize();
                        if collision_vec.x.is_nan() || collision_vec.y.is_nan() {
                            return None;
                        }
                        return Some(collision_vec * (sum_r - dist));
                    }
                }
                Shape::RectangeCollider { size } => {}
            },
            Shape::RectangeCollider { size } => match other {
                Shape::CircleCollider {
                    radius: other_radius,
                } => {}
                Shape::RectangeCollider { size: other_size } => {}
            },
        }
        None
    }

    fn get_collision_bounds(&self, pos: &Vector2, bounds: [f32; 4]) -> Option<Vector2> {
        let bounding_box = self.get_bounding_box(pos);
        if bounding_box[0].x < bounds[0] {
            return Some(Vector2::new(bounds[0] - bounding_box[0].x, 0f32));
        }
        if bounding_box[0].y < bounds[1] {
            return Some(Vector2::new(0f32, bounds[1] - bounding_box[0].y));
        }
        if bounding_box[1].x > bounds[2] {
            return Some(Vector2::new(bounds[2] - bounding_box[1].x, 0f32));
        }
        if bounding_box[1].y > bounds[3] {
            return Some(Vector2::new(0f32, bounds[3] - bounding_box[1].y));
        }
        None
    }

    fn get_bounding_box(&self, pos: &Vector2) -> [Vector2; 2] {
        let pos_clone = *pos;
        match self {
            Shape::CircleCollider { radius } => [
                pos_clone - Vector2::one() * (*radius),
                pos_clone + Vector2::one() * (*radius),
            ],
            Shape::RectangeCollider { size } => [pos_clone, pos_clone + *size],
        }
    }
}

pub fn get_aabb_union(first: &[Vector2; 2], second: &[Vector2; 2]) -> [Vector2; 2] {
    [
        Vector2::new(first[0].x.min(second[0].x), first[0].y.min(second[0].y)),
        Vector2::new(first[1].x.max(second[1].x), first[1].y.max(second[1].y)),
    ]
}

pub fn get_aabb_intersection(first: &[Vector2; 2], second: &[Vector2; 2]) -> [Vector2; 2] {
    [
        Vector2::new(first[0].x.max(second[0].x), first[0].y.max(second[0].y)),
        Vector2::new(first[1].x.min(second[1].x), first[1].y.min(second[1].y)),
    ]
}

pub fn is_aabb_colliding(first: &[Vector2; 2], second: &[Vector2; 2]) -> bool {
    first[1].x >= second[0].x
        && first[0].x <= second[1].x
        && first[1].y >= second[0].y
        && first[0].y <= second[1].y
}

pub fn is_aabb_inside(first: &[Vector2; 2], second: &[Vector2; 2]) -> bool {
    first[1].x >= second[1].x
        && first[0].x <= second[0].x
        && first[1].y >= second[1].y
        && first[0].y <= second[0].y
}
