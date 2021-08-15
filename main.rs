use ::std::cmp::min;
use raylib::prelude::*;
use rand::Rng;


const RADIUS: f32 = 10.0f32;
const COLLISION_FRICTION: f32 = 0.99f32;
const WALL_COLLISION_FRICTION: f32 = 0.5f32;
const FRICTION: f32 = 0.8f32;
const WINDOW_SIZE: [i32; 2] = [1024, 960];
const INITIAL_VELOCITY: f32 = 200f32;
const GRAVITY: f32 = 0.25f32;

#[derive(Debug, Clone)]
struct Particle{
    position: Vector2,
    velocity: Vector2,
    mass: f32,
    radius: f32,
    color: Color,
    frozen_time: i32
}

impl Particle {
    fn new(mass: f32) -> Particle{
        Particle{position: Vector2::new(0f32, 0f32), velocity: Vector2::new(0f32, 0f32), mass: mass, radius: RADIUS*mass, color: Color{r: 0, g: 0, b: 0, a: u8::MAX}, frozen_time: 0}
    }

    fn update(&mut self, delta: f32){
        self.position += self.velocity*delta;
        self.velocity *= f32::powf(FRICTION, delta);
        self.color = Color{r: (2f32*self.velocity.length()) as u8, g: 0, b: 0, a: u8::MAX};
    }

    fn collide_walls(&mut self){
        if self.position.x < self.radius{
            self.position.x = self.radius;
            self.velocity.x = -self.velocity.x*WALL_COLLISION_FRICTION;
        }
        if self.position.x > WINDOW_SIZE[0] as f32 - self.radius{
            self.position.x = WINDOW_SIZE[0] as f32 - self.radius;
            self.velocity.x = -self.velocity.x*WALL_COLLISION_FRICTION;
        }
        if self.position.y < self.radius{
            self.position.y = self.radius;
            self.velocity.y = -self.velocity.y*WALL_COLLISION_FRICTION;
        }
        if self.position.y > WINDOW_SIZE[1] as f32 - self.radius{
            self.position.y = WINDOW_SIZE[1] as f32 - self.radius;
            self.velocity.y = -self.velocity.y*WALL_COLLISION_FRICTION;
        }
    }

    fn resolve_collision(&mut self, other: &mut Particle) -> bool{
        let colliding_with = self.check_collision(other);
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

    fn check_collision(&self, other: &Particle) -> Option<Vector2>{
        let collision_vec = self.position - other.position;
        if collision_vec.length() <= (self.radius + other.radius){
            return Some(collision_vec);
        }
        None
    }
}
    impl PartialEq for Particle {
        fn eq(&self, other: &Self) -> bool {
            self.position == other.position && self.velocity == other.velocity
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