use raylib::math::Vector2;
use specs::{Component, VecStorage};

pub const LAYERS: usize = 128;

/// Handles narrow phase collisions, and generating aabbs.
// implement bottom up collision caching if physics_collider is true
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Collider {
    pub shape: Shape,
    pub physics_collider: bool,
    pub collision_layers: [bool; LAYERS],
    pub collision_mask: [bool; LAYERS],
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

    pub fn get_bounding_box(&self, pos: &Vector2) -> AABB {
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
                Shape::RectangeCollider { size: _ } => todo!(),
            },
            Shape::RectangeCollider { size: _ } => match other {
                Shape::CircleCollider { radius: _ } => todo!(),
                Shape::RectangeCollider { size: _ } => todo!(),
            },
        }
        None
    }

    fn get_collision_bounds(&self, pos: &Vector2, bounds: [f32; 4]) -> Option<Vector2> {
        let bounding_box = self.get_bounding_box(pos);
        if bounding_box.lx < bounds[0] {
            return Some(Vector2::new(bounds[0] - bounding_box.lx, 0f32));
        }
        if bounding_box.ly < bounds[1] {
            return Some(Vector2::new(0f32, bounds[1] - bounding_box.ly));
        }
        if bounding_box.rx > bounds[2] {
            return Some(Vector2::new(bounds[2] - bounding_box.rx, 0f32));
        }
        if bounding_box.ry > bounds[3] {
            return Some(Vector2::new(0f32, bounds[3] - bounding_box.ry));
        }
        None
    }

    fn get_bounding_box(&self, pos: &Vector2) -> AABB {
        match self {
            Shape::CircleCollider { radius } => AABB {
                lx: pos.x,
                rx: pos.x + 2.0 * radius,
                ly: pos.y,
                ry: pos.y + 2.0 * radius,
            },
            Shape::RectangeCollider { size } => AABB {
                lx: pos.x,
                rx: pos.x + size.x,
                ly: pos.y,
                ry: pos.y + size.x,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AABB {
    pub lx: f32,
    pub rx: f32,
    pub ly: f32,
    pub ry: f32,
}

impl AABB {
    pub fn with_point(&self, other: &Vector2) -> AABB {
        AABB {
            lx: self.lx.min(other.x),
            rx: self.rx.max(other.x),
            ly: self.ly.min(other.y),
            ry: self.ry.max(other.y),
        }
    }

    pub fn get_union(&self, other: &AABB) -> AABB {
        AABB {
            lx: self.lx.min(other.lx),
            rx: self.rx.max(other.rx),
            ly: self.ly.min(other.ly),
            ry: self.ry.max(other.ry),
        }
    }

    pub fn get_intersection(&self, other: &AABB) -> AABB {
        AABB {
            lx: self.lx.max(other.lx),
            rx: self.rx.min(other.rx),
            ly: self.ly.max(other.ly),
            ry: self.ry.min(other.ry),
        }
    }

    pub fn get_dist(&self, other: &AABB) -> f32 {
        let center_x = (self.lx + self.rx) / 2.0;
        let center_y = (self.ly + self.ry) / 2.0;
        let width_x = self.rx - self.lx;
        let width_y = self.ry - self.ly;
        let other_center_x = (other.lx + other.rx) / 2.0;
        let other_center_y = (other.ly + other.ry) / 2.0;
        let other_width_x = other.rx - other.lx;
        let other_width_y = other.ry - other.ly;
        let dx = (center_x - other_center_x).abs() - (width_x / 2.0 + other_width_x / 2.0);
        let dy = (center_y - other_center_y).abs() - (width_y / 2.0 + other_width_y / 2.0);
        (dx * dx).copysign(dx) + (dy * dy).copysign(dy)
    }

    pub fn is_colliding(&self, other: &AABB) -> bool {
        self.rx >= other.lx && self.lx <= other.rx && self.ry >= other.ly && self.ly <= other.ry
    }

    pub fn is_colliding_with_map(&self, other: &AABB, map: [bool; 4]) -> bool {
        (map[0] || self.rx >= other.lx)
            && (map[1] || self.lx <= other.rx)
            && (map[2] || self.ry >= other.ly)
            && (map[3] || self.ly <= other.ry)
    }

    pub fn contains(&self, other: &AABB) -> bool {
        self.rx >= other.rx && self.lx <= other.lx && self.ry >= other.ry && self.ly <= other.ly
    }
}
