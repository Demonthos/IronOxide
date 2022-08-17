use crate::COLLISION_FRICTION;
use crate::FRICTION;
use raylib::core::math::Vector2;
use specs::{Component, VecStorage};

/// Contains information about the mass and velocity of an entity.
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Physics {
    pub velocity: Vector2,
    mass: f32,
}

impl Physics {
    /// Creates a new Physics component.
    pub fn new(mass: f32) -> Physics {
        Physics {
            velocity: Vector2::new(0f32, 0f32),
            mass,
        }
    }

    /// Update a position with the physics system.
    pub fn update(&mut self, pos: &mut Vector2, delta: f32) {
        *pos += self.velocity * delta;
        self.velocity *= f32::powf(FRICTION, delta);
    }

    /// Resolve a collision between two entities
    pub fn resolve_collision(
        &mut self,
        pos: &mut Vector2,
        other_pos: &mut Vector2,
        other_physics: &mut Physics,
        overlap_vec: Vector2,
    ) {
        if overlap_vec.x == 0f32 || overlap_vec.y == 0f32 {
            return self.resolve_collision_aabb(pos, other_pos, other_physics, overlap_vec);
        }

        let diff = *pos - *other_pos;

        let normed = diff.normalized();
        let normed_sq = normed * normed;
        let div = normed_sq / diff;

        // if the distence is zero, the vector will be NaN
        // not sure if this is the best way to handle this, but it works
        if div.x.is_nan() || div.y.is_nan() {
            return;
        }

        let dot_prod = (self.velocity - other_physics.velocity).dot(diff);
        let new_vel = div * dot_prod;

        *pos -= overlap_vec / 2f32;
        let m = (2f32 * other_physics.mass) / (other_physics.mass + self.mass);
        let force = new_vel * m * COLLISION_FRICTION;
        self.velocity -= force;

        *other_pos += overlap_vec / 2f32;
        let m = (2f32 * self.mass) / (self.mass + other_physics.mass);
        let force = new_vel * m * COLLISION_FRICTION;
        other_physics.velocity += force;
    }

    /// Resolve a collision that happens between two axis aligned rectangles.
    fn resolve_collision_aabb(
        &mut self,
        pos: &mut Vector2,
        other_pos: &mut Vector2,
        other_physics: &mut Physics,
        overlap_vec: Vector2,
    ) {
        *pos -= overlap_vec / 2f32;
        *other_pos += overlap_vec / 2f32;
        let m1 = (2f32 * self.mass) / (self.mass + other_physics.mass);
        let m2 = (2f32 * other_physics.mass) / (self.mass + other_physics.mass);
        if overlap_vec.x == 0f32 {
            let other_vel = other_physics.velocity.y;
            other_physics.velocity.y = self.velocity.y * m2 * COLLISION_FRICTION;
            self.velocity.y = other_vel * m1 * COLLISION_FRICTION;
        } else {
            let other_vel = other_physics.velocity.x;
            other_physics.velocity.x = self.velocity.x * m2 * COLLISION_FRICTION;
            self.velocity.x = other_vel * m1 * COLLISION_FRICTION;
        }
    }

    /// Resolve a collision by only moving one entity.
    pub fn resolve_collision_single(
        &mut self,
        pos: &mut Vector2,
        other_pos: &Vector2,
        other_physics: &Physics,
        overlap_vec: Vector2,
    ) {
        if overlap_vec.x == 0f32 || overlap_vec.y == 0f32 {
            return self.resolve_collision_simple_aabb(pos, other_pos, other_physics, overlap_vec);
        }

        let dif = *pos - *other_pos;

        let normed = dif.normalized();
        let normed_sq = normed * normed;
        let div = normed_sq / dif;

        // not sure if this is the best way to handle this, but it works
        if div.x.is_nan() || div.y.is_nan() {
            return;
        }

        let dot_prod = (self.velocity - other_physics.velocity).dot(dif);
        let new_vel = div * dot_prod;

        *pos -= overlap_vec / 2f32;
        let m = (2f32 * other_physics.mass) / (other_physics.mass + self.mass);
        let force = new_vel * m * COLLISION_FRICTION;
        self.velocity -= force;
    }

    /// Resolve a collision that happens between two axis aligned rectangles by only moving one entity.
    fn resolve_collision_simple_aabb(
        &mut self,
        pos: &mut Vector2,
        _other_pos: &Vector2,
        other_physics: &Physics,
        overlap_vec: Vector2,
    ) {
        *pos -= overlap_vec / 2f32;
        let m1 = (2f32 * self.mass) / (self.mass + other_physics.mass);
        if overlap_vec.x == 0f32 {
            let other_vel = other_physics.velocity.y;
            self.velocity.y = other_vel * m1 * COLLISION_FRICTION;
        } else {
            let other_vel = other_physics.velocity.x;
            self.velocity.x = other_vel * m1 * COLLISION_FRICTION;
        }
    }

    /// Resolve a collision with a wall.
    pub fn collide_bound(&mut self, position: &mut Vector2, collision_vec: Vector2) {
        if collision_vec.x != 0f32 {
            position.x += collision_vec.x;
            self.velocity.x = -self.velocity.x;
        } else {
            position.y += collision_vec.y;
            self.velocity.y = -self.velocity.y;
        }
    }
}

impl PartialEq for Physics {
    fn eq(&self, other: &Self) -> bool {
        self.mass == other.mass && self.velocity == other.velocity
    }
}
