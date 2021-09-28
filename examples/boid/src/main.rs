extern crate iron_oxide;

use iron_oxide::Builder;
use iron_oxide::IndexedParallelIterator;
use iron_oxide::IntoParallelRefMutIterator;
use iron_oxide::ParallelIterator;
use iron_oxide::RaylibDraw;
use iron_oxide::WorldExt;
use std::collections::HashSet;

use iron_oxide::Color;

use iron_oxide::Join;

const INITIAL_VELOCITY: f32 = 400f32;
const RADIUS: f32 = 5.0f32;
const DEBUG_BVH: bool = true;

struct UpdateVelocity;

impl<'a> iron_oxide::System<'a> for UpdateVelocity {
    type SystemData = (
        iron_oxide::WriteStorage<'a, iron_oxide::utils::Position>,
        iron_oxide::ReadStorage<'a, iron_oxide::collider::Collider>,
        iron_oxide::WriteStorage<'a, iron_oxide::physics::Physics>,
        iron_oxide::ReadStorage<'a, iron_oxide::utils::Collisions>,
        iron_oxide::Read<'a, [i32; 2]>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let size = data.4;
        let mut entity_data: Vec<_> = (&mut data.0, &data.1, &mut data.2, &data.3)
            .join()
            .collect();

        // costly
        let old_positions: Vec<iron_oxide::Vector2> =
            (&entity_data).iter().map(|t| t.0 .0).collect();
        let old_physics: Vec<iron_oxide::physics::Physics> =
            (&entity_data).iter().map(|t| t.2.clone()).collect();

        entity_data.par_iter_mut().enumerate().for_each(|(i, p)| {
            let old_pos = &old_positions[i];
            let bb = p.1.get_bounding_box(old_pos);
            let collisions = &p.3 .0;

            if !collisions.is_empty() {
                let sum_pos_o = collisions
                    .iter()
                    .map(|i| old_positions[(*i) as usize])
                    .reduce(|i1, i2| i1 + i2);

                let close_vec: Vec<_> = collisions
                    .iter()
                    .map(|i| old_positions[(*i) as usize])
                    .filter_map(|position| {
                        let d = position.distance_to(*old_pos);
                        if d < (bb[1].x - bb[0].x) / 3.0 {
                            Some((*old_pos - position) / d)
                        } else {
                            None
                        }
                    })
                    .collect();

                let sum_close_o = close_vec.iter().copied().reduce(|i1, i2| (i1 + i2));

                let sum_vel_o = collisions
                    .iter()
                    .map(|i| old_physics[(*i) as usize].velocity)
                    .reduce(|i1, i2| i1 + i2);

                if let Some(sum_vel) = sum_vel_o {
                    p.2.velocity += sum_vel.normalized() * 4.0;
                }

                if let Some(sum_pos) = sum_pos_o {
                    let dif_pos = *old_pos - (sum_pos / collisions.len() as f32);

                    if dif_pos.length_sqr() > 0.0 {
                        p.2.velocity -= dif_pos.normalized() * 4.0;
                    }
                }

                if let Some(sum_close) = sum_close_o {
                    if sum_close.length_sqr() > 0.0 {
                        p.2.velocity += sum_close.normalized() * 5.0;
                    }
                }
            }

            if p.0 .0.x < 0.0 {
                p.0 .0.x = size[0] as f32;
            }

            if p.0 .0.x > size[0] as f32 {
                p.0 .0.x = 0.0;
            }

            if p.0 .0.y < 0.0 {
                p.0 .0.y = size[1] as f32;
            }

            if p.0 .0.y > size[1] as f32 {
                p.0 .0.y = 0.0;
            }

            p.2.velocity.normalize();
            p.2.velocity *= 100f32;
        });
    }
}

struct EntCount(usize);

fn main() {
    let mut builder = iron_oxide::build();

    builder
        .3
        .add(UpdateVelocity, "update_velocity", &["collide_entities"]);

    let mut data = iron_oxide::init(builder);
    let timer = data.0.get_time();
    data.2.insert(EntCount(0));
    data.2.insert(timer);

    let mut rng = iron_oxide::rand::thread_rng();

    for _ in 0..500 {
        gen_enity(&mut data.2, &mut rng, &mut data.4);
    }

    while !data.0.window_should_close() {
        iron_oxide::update(&mut data, draw);
    }
}

fn draw(world: &mut iron_oxide::World, d: &mut iron_oxide::prelude::RaylibDrawHandle) {
    let rng = iron_oxide::rand::thread_rng();
    // if rl.is_key_pressed(iron_oxide::KeyboardKey::KEY_R) {
    //     entity_count = 0;
    //     world.delete_all();
    //     world.maintain();
    // }

    // if rl.is_key_pressed(iron_oxide::KeyboardKey::KEY_SPACE) {
    //     timer = rl.get_time();
    // }

    // let mouse_pos = rl.get_mouse_position();

    // {
    //     let mut size = world.write_resource::<[i32; 2]>();
    //     *size = [rl.get_screen_width(), rl.get_screen_height()]
    // }

    // {
    //     let mut system_data: (
    //         iron_oxide::WriteStorage<iron_oxide::physics::Physics>,
    //         iron_oxide::ReadStorage<iron_oxide::utils::Position>,
    //     ) = world.system_data();
    //     for (phys, pos) in (&mut system_data.0, &system_data.1).join() {
    //         if rl.is_mouse_button_down(iron_oxide::MouseButton::MOUSE_LEFT_BUTTON) {
    //             //     let mut vec_2d = (mouse_pos - pos.0).normalized() * 10000f32
    //             //         / ((mouse_pos.x - pos.0.x) * (mouse_pos.x - pos.0.x)
    //             //             + (mouse_pos.y - pos.0.y) * (mouse_pos.y - pos.0.y));
    //             //     let temp = vec_2d.x;
    //             //     vec_2d.x = -vec_2d.y;
    //             //     vec_2d.y = temp;
    //             //     phys.velocity += vec_2d;
    //             phys.velocity += (mouse_pos - pos.0).normalized() * 20.0;
    //         }
    //     }
    // }

    // let l_m_down = rl.is_mouse_button_down(iron_oxide::MouseButton::MOUSE_RIGHT_BUTTON);

    // if rl.get_fps() > 100 {
    //     // if rl.is_key_down(KeyboardKey::KEY_SPACE) {
    //     if rl.get_time() - timer > 0.01 {
    //         gen_enity()
    //     }
    // }

    {
        let mut system_data: iron_oxide::RenderingData = world.system_data();
        for data in (
            &mut system_data.0,
            &system_data.1,
            (&system_data.2).maybe(),
            (&system_data.3).maybe(),
        )
            .join()
        {
            let (r, pos, phys, col) = data;
            if let Some(p) = phys {
                match r {
                    iron_oxide::renderer::Renderer::CircleRenderer { radius: _, color } => {
                        *color = iron_oxide::Color::color_from_hsv(
                            p.velocity.angle_to(iron_oxide::Vector2::one()) * 2f32
                                / std::f32::consts::PI.to_radians(),
                            1.0,
                            1.0,
                        );
                    }
                    iron_oxide::renderer::Renderer::RectangeRenderer { size: _, color } => {
                        *color = Color::color_from_hsv(
                            p.velocity.angle_to(iron_oxide::Vector2::one()) * 2f32
                                / std::f32::consts::PI.to_radians(),
                            1.0,
                            1.0,
                        );
                    }
                }
            }
            // if l_m_down {
            if let Some(c) = col {
                let bb = c.get_bounding_box(&pos.0);
                let bb_size = bb[1] - bb[0];
                d.draw_rectangle_lines(
                    bb[0].x as i32,
                    bb[0].y as i32,
                    bb_size.x as i32,
                    bb_size.y as i32,
                    iron_oxide::Color::new(0, 255, 0, 100),
                )
            }
            // }
            // d.draw_circle_v(p.position, 10f32, Color::new(255, 0, 255, 0));
        }

        if DEBUG_BVH {
            let bvh_read: iron_oxide::Read<Option<iron_oxide::bvh::BVHTree>> = world.system_data();
            if let Some(bvh_root) = &*bvh_read {
                for node in bvh_root.get_children() {
                    let rect;
                    match node {
                        iron_oxide::bvh::Node::Branch(bb, _) => rect = bb,
                        iron_oxide::bvh::Node::Fruit(bb, _, _) => rect = bb,
                    }
                    d.draw_rectangle(
                        rect[0].x as i32,
                        rect[0].y as i32,
                        (rect[1].x - rect[0].x) as i32,
                        (rect[1].y - rect[0].y) as i32,
                        iron_oxide::Color::new(255, 0, 0, 5),
                    );
                }
            }
        }
    }
    d.draw_text(
        format!("{:?}", { world.read_resource::<EntCount>().0 }).as_str(),
        0,
        20,
        20,
        iron_oxide::Color::BLACK,
    );
}

fn gen_enity(
    world: &mut iron_oxide::World,
    rng: &mut impl iron_oxide::rand::Rng,
    bvh_update: &mut f32,
) {
    {
        let x_size;
        let y_size;
        {
            let size = world.read_resource::<[i32; 2]>();
            x_size = size[0];
            y_size = size[1];
        }
        let radius = RADIUS;
        let position = iron_oxide::Vector2::new(
            rng.gen::<f32>() * x_size as f32,
            rng.gen::<f32>() * y_size as f32,
        );
        let mut particle_physics = iron_oxide::physics::Physics::new(radius);
        let mut rand_vec = iron_oxide::Vector2::new(0f32, 0f32);
        while rand_vec.length_sqr() == 0f32 {
            rand_vec = iron_oxide::Vector2::new(
                1.0 - 2.0 * rng.gen::<f32>(),
                1.0 - 2.0 * rng.gen::<f32>(),
            );
        }
        rand_vec.normalize();
        rand_vec.scale(INITIAL_VELOCITY);
        particle_physics.velocity = rand_vec;
        {
            world.write_resource::<EntCount>().0 += 1;
        }
        let collider = iron_oxide::collider::Collider {
            shape: iron_oxide::collider::Shape::CircleCollider {
                radius: radius * 1.0,
            },
            physics_collider: true,
        };

        let e = world
            .create_entity()
            .with(iron_oxide::utils::Position(position))
            .with(particle_physics)
            .with(collider.clone())
            // .with(iron_oxide::renderer::Renderer::RectangeRenderer {
            //     size: iron_oxide::Vector2::new(radius * 2f32, radius * 2f32),
            //     color: Color::new(0, 0, 0, 255),
            // })
            .with(iron_oxide::utils::Collisions(Vec::new()))
            .with(iron_oxide::renderer::Renderer::CircleRenderer {
                radius,
                color: Color::new(0, 0, 0, 255),
            })
            .build();

        iron_oxide::utils::register_ent(
            (
                &collider,
                position,
                collider.get_bounding_box(&position),
                e.id(),
                HashSet::new(),
            ),
            world,
            bvh_update,
        );
    }
}