use std::collections::HashMap;
use std::any::{Any, TypeId};
use ::std::cmp::min;
use raylib::prelude::*;
use rand::Rng;

const RADIUS: f32 = 10.0f32;
const COLLISION_FRICTION: f32 = 0.99f32;
const WALL_COLLISION_FRICTION: f32 = 0.5f32;
const FRICTION: f32 = 0.8f32;
const WINDOW_SIZE: [i32; 2] = [600, 600];
const INITIAL_VELOCITY: f32 = 200f32;
const GRAVITY: f32 = 0.25f32;

struct CircleCollider{
    radius: f32,
}

impl CircleCollider {
    fn collide_walls(&mut self, pos: &Vector2){
        if pos.x < self.radius{
            pos.x = self.radius;
        }
        if pos.x > WINDOW_SIZE[0] as f32 - self.radius{
            pos.x = WINDOW_SIZE[0] as f32 - self.radius;
        }
        if pos.y < self.radius{
            pos.y = self.radius;
        }
        if pos.y > WINDOW_SIZE[1] as f32 - self.radius{
            pos.y = WINDOW_SIZE[1] as f32 - self.radius;
        }
    }

    fn check_collision(&self, pos: &Vector2, other: &CircleCollider, other_pos: &Vector2) -> Option<Vector2>{
        let collision_vec = pos - other_pos;
        if collision_vec.length() <= (self.radius + other.radius){
            return Some(collision_vec);
        }
        None
    }
}

#[derive(Debug, Clone)]
struct Physics {
    velocity: Vector2,
    mass: f32,
    frozen_time: i32
}

impl Physics {
    fn new(mass: f32) -> Physics{
        Physics{velocity: Vector2::new(0f32, 0f32), mass: mass, frozen_time: 0}
    }

    fn update(&mut self, pos: &mut Vector2, delta: f32){
        *pos += self.velocity*delta;
        self.velocity *= f32::powf(FRICTION, delta);
    }

    fn resolve_collision(&mut self, pos: &Vector2, other: &mut Physics, other_pos: &Vector2) -> bool{
        let colliding_with = self.check_collision(pos, other, other_pos);
        if colliding_with != None {
            let collision_vec = colliding_with.unwrap();
            let normalized_vec = collision_vec.normalized();
            let overlap_vec = normalized_vec*(collision_vec.length() - (self.radius + other.radius));
            // // add a bit more to make sure it doesn't overlap next frame
            // overlap_vec += overlap_vec.normalized();

            self.position -= overlap_vec/2f32;
            let m = (2f32*other.mass)/(other.mass + self.mass);
            let normed = (self.position - other.position).normalized();
            let dot_prod = (self.velocity - other.velocity).dot(self.position - other.position);
            let new_vel = ((normed*normed)/(self.position - other.position))*dot_prod;

            self.velocity -= new_vel*m;
            self.velocity *= COLLISION_FRICTION;

            other.position += overlap_vec/2f32;
            let m = (2f32*self.mass)/(self.mass + other.mass);
            other.velocity += new_vel*m;
            other.velocity *= COLLISION_FRICTION;

            return true
        }
        false
    }
}
    impl PartialEq for Physics {
        fn eq(&self, other: &Self) -> bool {
            self.position == other.position && self.velocity == other.velocity
        }
    }

#[derive(Debug)]
struct Entity {
    position: Vector2,
    components: HashMap<TypeId, Box<dyn Any>>
}

impl Entity {
    fn add_component(&mut self, new_component: Box<dyn Any>) {
        self.components.insert(new_component.type_id(), new_component);
    }
}

struct Particle{
    entity: Entity,
}

impl Particle {
    fn new(physicsComponent: Physics, radius: f32, color: Color) -> Particle{
        let new_particle = Particle{Entity{position, HashMap::new()}};
        new_particle.entity.add_component(physicsComponent);
        new_particle.entity.add_component();
        new_particle
    }
    
    fn update(&mut self){
        self.color = Color{r: (2f32*self.velocity.length()) as u8, g: 0, b: 0, a: u8::MAX};
    }
}
    impl PartialEq for Particle {
        fn eq(&self, other: &Self) -> bool {
            self.physics == other.physics && self.collider == other.collider && self.color == other.color
        }
    }


fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_SIZE[0], WINDOW_SIZE[1])
        .title("Hello, World")
        .build();

    let mut particles: Vec<Particle> = Vec::new();
    let mut timer = rl.get_time();
    let mut rng = rand::thread_rng();

    while !rl.window_should_close() {
        let mouse_pos = rl.get_mouse_position();

        let delta = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE){
            timer = rl.get_time();
        }
        // if rl.is_key_down(KeyboardKey::KEY_SPACE) && particles.len() < 400{
        if particles.len() < 400{
            if rl.get_time() - timer > 0.05{
                let mut p = Particle::new(0.5f32 + ((rng.gen::<u8>()%32) as f32)/16f32);
                let mut rand_vec = Vector2::new(0f32, 0f32);
                while rand_vec.length_sqr() == 0f32{
                    rand_vec = Vector2::new(rng.gen::<f32>(), rng.gen::<f32>());
                }
                rand_vec.normalize();
                rand_vec.scale(INITIAL_VELOCITY);
                p.velocity = rand_vec;
                p.position.x = rng.gen::<f32>()*WINDOW_SIZE[0] as f32;
                particles.push(p);
                timer = rl.get_time();
            }
        }

        for mut p in &mut particles{
            if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON){
                p.velocity += (mouse_pos - p.position).normalized();
            }
            p.velocity.y += GRAVITY;
        }

        if particles.len() > 0{
            for i in 1..particles.len()+1{
                let (l, r) = particles.split_at_mut(i);
                let p = &mut l[l.len()-1];
                p.update(delta);
                p.collide_walls();

                for mut p2 in &mut *r{
                    if p.resolve_collision(&mut p2) {
                        p.collide_walls();
                        p2.collide_walls();
                        // break;
                    }
                }

                for mut p2 in &mut *r{
                    if p.check_collision(p2) != None{
                        p.frozen_time += 1;
                        p.color.g = min(p.frozen_time, 255) as u8;
                        p2.frozen_time += 1;
                        p2.color.g = min(p2.frozen_time, 255) as u8;
                    }
                }

                if p.color.g == 0{
                    p.frozen_time = 0;
                }
            }
        }

        let mut d = rl.begin_drawing(&thread);

        for p in &particles{
            d.draw_circle_v(p.position, p.radius, p.color);
        }

        d.draw_fps(0, 0);
        d.draw_text(format!("{:?}", particles.len()).as_str(), 0, 20, 20, Color::BLACK);
        // println!("{:?}", particles);
        d.clear_background(Color::WHITE);
    }
}