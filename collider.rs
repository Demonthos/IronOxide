use raylib::math::Vector2;

pub enum Collider {
    CircleCollider { radius: f32 },
    RectangeCollider { size: Vector2 },
}

impl Collider {
    pub fn get_collision(
        &self,
        pos: &Vector2,
        other_pos: &Vector2,
        other: &Collider,
    ) -> Option<Vector2> {
        match self {
            Collider::CircleCollider { radius } => match other {
                Collider::CircleCollider {
                    radius: other_radius,
                } => {
                    let collision_vec = (other_pos.clone() + Vector2::one() * (*other_radius))
                        - (pos.clone() + Vector2::one() * (*radius));
                    let dist = collision_vec.length();
                    let sum_r = radius + other_radius;
                    if dist <= sum_r {
                        return Some(collision_vec.normalized() * (sum_r - dist));
                    }
                }
                Collider::RectangeCollider { size } => {}
            },
            Collider::RectangeCollider { size } => {}
        }
        None
    }

    pub fn get_collision_bounds(&self, pos: &Vector2, bounds: [f32; 4]) -> Option<Vector2> {
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

    pub fn get_bounding_box(&self, pos: &Vector2) -> [Vector2; 2] {
        let pos_clone = pos.clone();
        match self {
            Collider::CircleCollider { radius } => {
                [pos_clone, pos_clone + Vector2::one() * (*radius) * 2f32]
            }
            Collider::RectangeCollider { size } => [pos_clone, pos_clone + size.clone()],
        }
    }
}

pub fn get_aabb_union(first: &[Vector2; 2], second: &[Vector2; 2]) -> [Vector2; 2] {
    [
        Vector2::new(first[0].x.min(second[0].x), first[0].y.min(second[0].y)),
        Vector2::new(first[1].x.max(second[1].x), first[1].y.max(second[1].y)),
    ]
}
