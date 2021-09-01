use hecs::World;
use rand::Rng;
use raylib::prelude::*;
use std::collections::HashSet;

mod bvh;
mod collider;
mod physics;
mod renderer;
mod utils;
// mod tests;

const RADIUS: f32 = 10.0f32;
const COLLISION_FRICTION: f32 = 0.998f32;
// const COLLISION_FRICTION: f32 = 0.99f32;
const FRICTION: f32 = 0.998f32;
// const FRICTION: f32 = 1f32;
const WINDOW_SIZE: [i32; 2] = [1400, 1000];
const SCREEN_BOUNDS: [f32; 4] = [0f32, 0f32, WINDOW_SIZE[0] as f32, WINDOW_SIZE[1] as f32];
const INITIAL_VELOCITY: f32 = 400f32;
const GRAVITY: f32 = 1f32;
// const GRAVITY: f32 = 0f32;
const MIN_BHV_UPDATE_TIME: f32 = 0.1f32;

#[derive(Clone)]
struct Particle {
    position: Vector2,
    physics: physics::Physics,
    collider: collider::Collider,
    renderer: renderer::Renderer,
}

impl Particle {
    fn new(position: Vector2, radius: f32) -> Particle {
        Particle {
            position,
            physics: physics::Physics::new(radius),
            collider: collider::Collider::CircleCollider { radius },
            renderer: renderer::Renderer::CircleRenderer {
                radius,
                color: Color::new(0, 0, 0, 255),
            },
        }
        // Particle {
        //     position: position,
        //     physics: physics::Physics::new(radius),
        //     collider: collider::Collider::RectangeCollider {
        //         size: Vector2::new(radius * 2f32, radius * 2f32),
        //     },
        //     renderer: renderer::Renderer::RectangeRenderer {
        //         size: Vector2::new(radius * 2f32, radius * 2f32),
        //         color: Color::new(0, 0, 0, 255),
        //     },
        // }
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
    // let mut world = World::new();

    let mut time_since_bvh_update = 0f32;
    let mut bvh_tree = None;

    let mut hs1 = HashSet::new();
    hs1.insert(0);
    let mut hs2 = HashSet::new();
    hs2.insert(1);

    while !rl.window_should_close() {
        let mouse_pos = rl.get_mouse_position();

        let delta = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            particles = Vec::new();
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            timer = rl.get_time();
        }

        if rl.get_fps() > 50 {
            // if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            if rl.get_time() - timer > 0.01 {
                let mut p = Particle::new(
                    Vector2::new(rng.gen::<f32>() * WINDOW_SIZE[0] as f32, 0f32),
                    5f32 + RADIUS * ((rng.gen::<u8>() % 32) as f32) / 64f32,
                );
                let mut rand_vec = Vector2::new(0f32, 0f32);
                while rand_vec.length_sqr() == 0f32 {
                    rand_vec = Vector2::new(rng.gen::<f32>(), rng.gen::<f32>());
                }
                rand_vec.normalize();
                rand_vec.scale(INITIAL_VELOCITY);
                p.physics.velocity = rand_vec;
                particles.push(p);
                time_since_bvh_update = 1f32 + MIN_BHV_UPDATE_TIME;
                timer = rl.get_time();
            }
        }

        for mut p in &mut particles {
            if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
                p.physics.velocity += (mouse_pos - p.position).normalized() * 10f32;
            }
            p.physics.velocity.y += GRAVITY;
            p.physics.velocity *= FRICTION;
            match p.renderer {
                renderer::Renderer::CircleRenderer {
                    radius: _,
                    ref mut color,
                } => {
                    color.g = 0;
                    color.r = 0;
                }
                renderer::Renderer::RectangeRenderer {
                    size: _,
                    ref mut color,
                } => {
                    color.g = 0;
                    color.r = 0;
                }
            }
        }

        let length = particles.len();

        if !particles.is_empty() {
            time_since_bvh_update += delta;

            for p in &mut particles {
                p.physics.update(&mut p.position, delta);
            }

            // particles.shuffle(&mut rng);
            // costly
            let old_particles = particles.clone();

            if time_since_bvh_update > MIN_BHV_UPDATE_TIME {
                bvh_tree = Some(create_bvh(&old_particles));
                // println!("{:?}", time_since_bvh_update);
                time_since_bvh_update = 0f32;
            } else if let Some(ref mut bvh) = bvh_tree {
                for i in 0..(particles.len() - 1) {
                    let o = &old_particles[i];
                    let n = &particles[i];
                    bvh.update(
                        (o.collider.get_bounding_box(&o.position), i as u32),
                        (n.collider.get_bounding_box(&n.position), i as u32),
                    );
                }
            }

            if let Some(ref bvh) = bvh_tree {
                // 1323 50fps
                // 5193 50fps
                // make sure collisions are not being resolved twice!!!
                for i in 1..particles.len() + 1 {
                    let hs = if i < length / 2 { &hs1 } else { &hs2 };
                    // let hs = &hs1;

                    let (l, r) = particles.split_at_mut(i);
                    let p = &mut l[l.len() - 1];
                    let old_p = &old_particles[i - 1];
                    let collisions =
                        bvh.query_rect(old_p.collider.get_bounding_box(&old_p.position), Some(hs));

                    for p2_index in &collisions {
                        if p2_index >= &&(i as u32) {
                            // println!("{:?}", p2_index);
                            let p2m = &mut r[(**p2_index) as usize - i];
                            let p2 = &old_particles[(**p2_index) as usize];
                            let overlap_vec = old_p.collider.get_collision(
                                &old_p.position,
                                &p2.position,
                                &p2.collider,
                            );
                            if let Some(unwraped) = overlap_vec {
                                p.physics.resolve_collision(
                                    &mut p.position,
                                    &mut p2m.position,
                                    &mut p2m.physics,
                                    unwraped,
                                );
                            }
                        }
                    }
                    let overlap_vec = p.collider.get_collision_bounds(&p.position, SCREEN_BOUNDS);
                    if let Some(unwraped) = overlap_vec {
                        p.physics.collide_bound(&mut p.position, unwraped);
                    }
                }
            }
        }

        let l_m_down = rl.is_mouse_button_down(MouseButton::MOUSE_RIGHT_BUTTON);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        for (i, p) in particles.iter_mut().enumerate() {
            match p.renderer {
                renderer::Renderer::CircleRenderer {
                    radius: _,
                    ref mut color,
                } => {
                    color.g = if i < length / 2 { 255 } else { 0 };
                    color.r = (p.physics.velocity.length() * 0.75f32) as u8;
                }
                renderer::Renderer::RectangeRenderer {
                    size: _,
                    ref mut color,
                } => {
                    color.g = if i < length / 2 { 255 } else { 0 };
                    color.r = (p.physics.velocity.length() * 0.75f32) as u8;
                }
            }
            p.renderer.render(&mut d, &p.position);
            if l_m_down {
                let bb = p.collider.get_bounding_box(&p.position);
                let bb_size = bb[1] - bb[0];
                d.draw_rectangle_lines(
                    bb[0].x as i32,
                    bb[0].y as i32,
                    bb_size.x as i32,
                    bb_size.y as i32,
                    Color::new(0, 255, 0, 100),
                )
            }
            // d.draw_circle_v(p.position, 10f32, Color::new(255, 0, 255, 0));
        }

        d.draw_fps(0, 0);
        d.draw_text(
            format!("{:?}", particles.len()).as_str(),
            0,
            20,
            20,
            Color::BLACK,
        );
    }
}

fn create_bvh(particles: &[Particle]) -> bvh::BVHTree {
    let mut data = Vec::new();

    let length = particles.len();

    for (i, p) in particles.iter().enumerate() {
        let mut hs = HashSet::new();
        hs.insert(if i < length / 2 { 0 } else { 1 });
        // hs.insert(0);
        data.push((
            &p.collider,
            p.position,
            p.collider.get_bounding_box(&p.position),
            i as u32,
            hs,
        ));
    }

    bvh::BVHTree::new(data)
}
