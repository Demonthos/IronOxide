use hecs::World;
use rand::Rng;
use raylib::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

mod bvh;
mod collider;
mod physics;
mod renderer;
mod utils;
// mod tests;

const RADIUS: f32 = 10.0f32;
const COLLISION_FRICTION: f32 = 0.98f32;
const WALL_COLLISION_FRICTION: f32 = 0.5f32;
const FRICTION: f32 = 0.998f32;
const WINDOW_SIZE: [i32; 2] = [800, 800];
const SCREEN_BOUNDS: [f32; 4] = [0f32, 0f32, WINDOW_SIZE[0] as f32, WINDOW_SIZE[1] as f32];
const INITIAL_VELOCITY: f32 = 200f32;
const GRAVITY: f32 = 0.5f32;

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
            position: position,
            physics: physics::Physics::new(radius),
            collider: collider::Collider::CircleCollider { radius },
            renderer: renderer::Renderer::CircleRenderer {
                radius: radius,
                color: Color::new(0, 0, 0, 255),
            },
        }
        // Particle{position: position, physics: physics::Physics::new(radius), collider: collider::Collider::RectangeCollider{size: Vector2::new(radius*2f32, radius*2f32)}, renderer: renderer::Renderer::RectangeRenderer{size: Vector2::new(radius*2f32, radius*2f32), color: Color::new(0, 0, 0, 255)}}
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

    while !rl.window_should_close() {
        let mouse_pos = rl.get_mouse_position();

        let delta = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            timer = rl.get_time();
        }
        // if particles.len() < 400{
        if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            if rl.get_time() - timer > 0.05 {
                let mut p = Particle::new(
                    Vector2::new(rng.gen::<f32>() * WINDOW_SIZE[0] as f32, 0f32),
                    5f32 + RADIUS * ((rng.gen::<u8>() % 32) as f32) / 32f32,
                );
                let mut rand_vec = Vector2::new(0f32, 0f32);
                while rand_vec.length_sqr() == 0f32 {
                    rand_vec = Vector2::new(rng.gen::<f32>(), rng.gen::<f32>());
                }
                rand_vec.normalize();
                rand_vec.scale(INITIAL_VELOCITY);
                p.physics.velocity = rand_vec;
                particles.push(p);
                timer = rl.get_time();
            }
        }

        for mut p in &mut particles {
            if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
                p.physics.velocity += (mouse_pos - p.position).normalized() * 2f32;
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

        if particles.len() > 0 {
            // for i in 1..particles.len() + 1 {
            //     let (l, r) = particles.split_at_mut(i);
            //     let p = &mut l[l.len() - 1];
            //     p.physics.update(&mut p.position, delta);

            //     for p2 in &mut *r {
            //         let overlap_vec =
            //             p.collider
            //                 .get_collision(&p.position, &p2.position, &p2.collider);
            //         if let Some(unwraped) = overlap_vec {
            //             p.physics.resolve_collision(
            //                 &mut p.position,
            //                 &mut p2.position,
            //                 &mut p2.physics,
            //                 unwraped,
            //             );
            //             // break;
            //         }
            //     }

            //     let overlap_vec = p.collider.get_collision_bounds(&p.position, SCREEN_BOUNDS);
            //     if let Some(unwraped) = overlap_vec {
            //         p.physics.collide_bound(&mut p.position, unwraped);
            //     }
            // }

            let mut data = Vec::new();

            // costly
            let old_particles = particles.clone();

            for (i, p) in old_particles.iter().enumerate() {
                data.push((&p.collider, p.position.clone(), i));
            }

            let bvh_tree = bvh::BVHTree::new(data);
            // 815 50fps static or 815 moving
            for i in 1..particles.len() + 1 {
                let (l, r) = particles.split_at_mut(i);
                let p = &mut l[l.len() - 1];
                p.physics.update(&mut p.position, delta);
                for (_, p2_index) in bvh_tree.query_rect(p.collider.get_bounding_box(&p.position)) {
                    if p2_index >= &i {
                        // println!("{:?}", p2_index);
                        let p2 = &mut r[*p2_index - i];
                        let overlap_vec =
                            p.collider
                                .get_collision(&p.position, &p2.position, &p2.collider);
                        if let Some(unwraped) = overlap_vec {
                            p.physics.resolve_collision(
                                &mut p.position,
                                &mut p2.position,
                                &mut p2.physics,
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

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        for p in &mut particles {
            match p.renderer {
                renderer::Renderer::CircleRenderer {
                    radius: _,
                    ref mut color,
                } => {
                    color.r = (p.physics.velocity.length() * 0.75f32) as u8;
                }
                renderer::Renderer::RectangeRenderer {
                    size: _,
                    ref mut color,
                } => {
                    color.r = (p.physics.velocity.length() * 0.75f32) as u8;
                }
            }
            p.renderer.render(&mut d, &p.position);
            let bb = p.collider.get_bounding_box(&p.position);
            let bb_size = bb[1] - bb[0];
            d.draw_rectangle_lines(
                bb[0].x as i32,
                bb[0].y as i32,
                bb_size.x as i32,
                bb_size.y as i32,
                Color::new(0, 255, 0, 100),
            )
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
