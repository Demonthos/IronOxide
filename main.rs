#[macro_use]
extern crate lazy_static;
use rand::Rng;
use raylib::prelude::*;
use rayon::prelude::*;
use specs::DispatcherBuilder;
use specs::{
    Builder, Entities, Join, ParJoin, Read, ReadStorage, System, World, WorldExt, Write,
    WriteStorage,
};
// use std::cmp::max;
// use std::cmp::min;
use std::collections::HashSet;

mod bvh;
mod collider;
mod physics;
mod renderer;
mod utils;
// mod tests;

const RADIUS: f32 = 5.0f32;
// const COLLISION_FRICTION: f32 = 0.998f32;
const COLLISION_FRICTION: f32 = 1f32;
// const FRICTION: f32 = 0.998f32;
const FRICTION: f32 = 1f32;
const INITIAL_VELOCITY: f32 = 400f32;
// const GRAVITY: f32 = 1f32;
const GRAVITY: f32 = 0f32;
// const MIN_BHV_UPDATE_TIME: f32 = 100f32;
const MIN_BHV_UPDATE_TIME: f32 = 0.15f32;
const WINDOW_SIZE: [i32; 2] = [1400, 1000];
lazy_static! {
    static ref HS1: HashSet<i8> = vec![0].into_iter().collect();
    static ref HS2: HashSet<i8> = vec![1].into_iter().collect();
}

type RenderingData<'a> = (
    WriteStorage<'a, renderer::Renderer>,
    ReadStorage<'a, utils::Position>,
    ReadStorage<'a, physics::Physics>,
    WriteStorage<'a, collider::Collider>,
);

type BvhData<'a> = (
    Entities<'a>,
    ReadStorage<'a, utils::Position>,
    ReadStorage<'a, collider::Collider>,
);

#[derive(Default)]
struct Delta(f32);

struct UpdatePhysics;

impl<'a> System<'a> for UpdatePhysics {
    type SystemData = (
        Write<'a, Option<bvh::BVHTree>>,
        Entities<'a>,
        ReadStorage<'a, collider::Collider>,
        Read<'a, Delta>,
        WriteStorage<'a, utils::Position>,
        WriteStorage<'a, physics::Physics>,
    );

    fn run(&mut self, (mut bvh_tree, ents, col, delta, mut pos, mut phys): Self::SystemData) {
        (&mut phys).par_join().for_each(|phys| {
            phys.velocity.y += GRAVITY;
            phys.velocity *= FRICTION;
        });

        // make this parrelel
        if let Some(ref mut bvh) = *bvh_tree {
            for (pos, phys, col_m, ent) in (&mut pos, &mut phys, (&col).maybe(), &ents).join() {
                let old_pos = pos.0;
                phys.update(&mut pos.0, delta.0);
                if let Some(col) = col_m {
                    bvh.update(
                        (col.get_bounding_box(&old_pos), ent.id() as u32),
                        (col.get_bounding_box(&pos.0), ent.id() as u32),
                    );
                }
            }
        }
    }
}

struct CollideBounds;

impl<'a> System<'a> for CollideBounds {
    type SystemData = (
        Read<'a, [i32; 2]>,
        WriteStorage<'a, utils::Position>,
        ReadStorage<'a, collider::Collider>,
        WriteStorage<'a, physics::Physics>,
    );

    fn run(&mut self, (size, mut pos, col, mut phys): Self::SystemData) {
        (&mut pos, &col, &mut phys)
            .par_join()
            .filter(|(_, col, _)| col.physics_collider)
            .for_each(|(pos, col, phys)| {
                let overlap_vec =
                    col.get_collision_bounds(&pos.0, [0.0, 0.0, size[0] as f32, size[1] as f32]);
                if let Some(unwraped) = overlap_vec {
                    phys.collide_bound(&mut pos.0, unwraped);
                }
            });
    }
}

struct CollideEnities;

impl<'a> System<'a> for CollideEnities {
    type SystemData = (
        Read<'a, Option<bvh::BVHTree>>,
        WriteStorage<'a, utils::Position>,
        ReadStorage<'a, collider::Collider>,
        WriteStorage<'a, physics::Physics>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let bvh_tree = data.0;
        let mut entity_data: Vec<(
            &mut utils::Position,
            &collider::Collider,
            &mut physics::Physics,
        )> = (&mut data.1, &data.2, &mut data.3)
            .join()
            .filter(|(_, col, _)| col.physics_collider)
            .collect();

        // costly
        let old_positions: Vec<Vector2> = (&entity_data).iter().map(|t| t.0 .0).collect();
        let old_physics: Vec<physics::Physics> =
            (&entity_data).iter().map(|t| t.2.clone()).collect();
        let old_collidors: Vec<collider::Collider> =
            (&entity_data).iter().map(|t| t.1.clone()).collect();

        if let Some(ref bvh) = *bvh_tree {
            // let mut d = Vec::new();
            // for (i, (e, old_pos)) in entity_data.into_iter().zip(old_positions).enumerate() {
            //     let hs = &*HS1;
            //     d.push((i as i32, e.1.get_bounding_box(&old_pos), Some(hs)));
            // }
            // bvh.query_rect_batched(&d);

            // for i in 1..entity_data.len() + 1 {
            //     let hs = &*HS1;

            //     let (l, r) = entity_data.split_at_mut(i);
            //     let p = &mut l[l.len() - 1];
            //     let old_pos = &old_positions[i - 1];
            //     let collisions = bvh.query_rect(p.1.get_bounding_box(&old_pos), Some(hs));

            //     for p2_index in &collisions {
            //         // make sure collisions are not handled twice
            //         if p2_index >= &(i as u32) {
            //             // println!("{:?}", p2_index);
            //             let p2m = &mut r[(*p2_index) as usize - i];
            //             let p2_pos = &old_positions[(*p2_index) as usize];
            //             let overlap_vec = p.1.get_collision(&old_pos, &p2_pos, &p2m.1);
            //             if let Some(unwraped) = overlap_vec {
            //                 p.2.resolve_collision(&mut p.0 .0, &mut p2m.0 .0, &mut p2m.2, unwraped);
            //             }
            //         }
            //     }
            // }

            entity_data.par_iter_mut().enumerate().for_each(|(i, p)| {
                let hs = &*HS1;
                let old_pos = &old_positions[i];
                let collisions = bvh.query_rect(p.1.get_bounding_box(old_pos), None);

                for p2_index in &collisions {
                    // if p2_index >= &(i as u32) {
                    // println!("{:?}", p2_index);
                    let p2_pos = &old_positions[(*p2_index) as usize];
                    let p2_phys = &old_physics[(*p2_index) as usize];
                    let p2_col = &old_collidors[(*p2_index) as usize];
                    let overlap_vec = p.1.get_collision(old_pos, p2_pos, p2_col);
                    if let Some(unwraped) = overlap_vec {
                        // println!("{:#?}, {:#?}, {:#?}", p.0 .0, p2_pos, unwraped);
                        // make sure collisions are not handled twice, but we calculate it twice
                        p.2.resolve_collision_single(&mut p.0 .0, p2_pos, p2_phys, unwraped);
                    }
                    // }
                }
            });
        }
    }
}

struct UpdateVelocity;

impl<'a> System<'a> for UpdateVelocity {
    type SystemData = (
        Read<'a, Option<bvh::BVHTree>>,
        WriteStorage<'a, utils::Position>,
        ReadStorage<'a, collider::Collider>,
        WriteStorage<'a, physics::Physics>,
        Read<'a, [i32; 2]>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let size = data.4;
        let bvh_tree = data.0;
        let mut entity_data: Vec<(
            &mut utils::Position,
            &collider::Collider,
            &mut physics::Physics,
        )> = (&mut data.1, &data.2, &mut data.3).join().collect();

        // costly
        let old_positions: Vec<Vector2> = (&entity_data).iter().map(|t| t.0 .0).collect();
        let old_physics: Vec<physics::Physics> =
            (&entity_data).iter().map(|t| t.2.clone()).collect();

        if let Some(ref bvh) = *bvh_tree {
            entity_data.par_iter_mut().enumerate().for_each(|(i, p)| {
                let hs = None;
                let old_pos = &old_positions[i];
                let bb = p.1.get_bounding_box(old_pos);
                let collisions: Vec<_> = bvh
                    .query_rect(bb, hs)
                    .into_iter()
                    .filter(|id| *id != i as u32)
                    .collect();

                if collisions.len() > 0 {
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
}

/// update loop
// 750 particles 50fps
// 2300 particles 50fps
fn main() {
    let (mut rl, thread) = raylib::init()
        .resizable()
        .transparent()
        // .undecorated()
        .size(WINDOW_SIZE[0], WINDOW_SIZE[1])
        .title("Hello, World")
        .build();

    let mut time_since_bvh_update = 0f32;
    let bvh_tree: Option<bvh::BVHTree> = None;

    let mut world = World::new();
    world.register::<utils::Position>();
    world.register::<physics::Physics>();
    world.register::<collider::Collider>();
    world.register::<renderer::Renderer>();
    world.insert(Delta(0.00));
    world.insert([rl.get_screen_width(), rl.get_screen_height()]);
    world.insert(bvh_tree);
    let mut dispatcher = DispatcherBuilder::new()
        .with(UpdatePhysics, "update_physics", &[])
        .with(UpdateVelocity, "update_velocity", &[])
        .with(CollideBounds, "collide_bounds", &["update_physics"])
        .with(CollideEnities, "collide_entities", &["update_physics"])
        // .with(HelloWorld, "hello_updated", &["update_pos"])
        .build();

    let mut timer = rl.get_time();
    let mut rng = rand::thread_rng();

    let mut entity_count = 0;

    let mut clear_once = true;

    while !rl.window_should_close() {
        superluminal_perf::begin_event("other");
        dispatcher.dispatch(&world);
        world.maintain();
        superluminal_perf::end_event();

        let mouse_pos = rl.get_mouse_position();

        {
            let mut delta = world.write_resource::<Delta>();
            *delta = Delta(rl.get_frame_time());
            time_since_bvh_update += delta.0;
        }

        {
            let mut size = world.write_resource::<[i32; 2]>();
            *size = [rl.get_screen_width(), rl.get_screen_height()]
        }

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            entity_count = 0;
            world.delete_all();
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            timer = rl.get_time();
        }

        if rl.get_fps() > 100 {
            // if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            if rl.get_time() - timer > 0.01 {
                let x_size;
                let y_size;
                {
                    let size = world.read_resource::<[i32; 2]>();
                    x_size = size[0];
                    y_size = size[1];
                }
                let radius = RADIUS;
                let position = Vector2::new(
                    rng.gen::<f32>() * x_size as f32,
                    rng.gen::<f32>() * y_size as f32,
                );
                let mut particle_physics = physics::Physics::new(radius);
                let mut rand_vec = Vector2::new(0f32, 0f32);
                while rand_vec.length_sqr() == 0f32 {
                    rand_vec =
                        Vector2::new(1.0 - 2.0 * rng.gen::<f32>(), 1.0 - 2.0 * rng.gen::<f32>());
                }
                rand_vec.normalize();
                rand_vec.scale(INITIAL_VELOCITY);
                particle_physics.velocity = rand_vec;
                entity_count += 1;
                let collider = collider::Collider {
                    shape: collider::Shape::CircleCollider {
                        radius: radius * 5.0,
                    },
                    physics_collider: false,
                };
                let e = world
                    .create_entity()
                    .with(utils::Position(position))
                    .with(particle_physics)
                    .with(collider.clone())
                    // .with(renderer::Renderer::RectangeRenderer {
                    //     size: Vector2::new(radius * 2f32, radius * 2f32),
                    //     color: Color::new(255, 255, 255, 255),
                    // })
                    .with(renderer::Renderer::CircleRenderer {
                        radius,
                        color: Color::new(255, 255, 255, 255),
                    })
                    .build();
                // time_since_bvh_update = 1f32 + MIN_BHV_UPDATE_TIME;
                {
                    let tuple_data = (
                        &collider,
                        position,
                        collider.get_bounding_box(&position),
                        e.id(),
                        HS1.clone(),
                    );
                    let mut bvh_write: Write<Option<bvh::BVHTree>> = world.system_data();
                    if let Some(ref mut bvh) = *bvh_write {
                        bvh.insert(&tuple_data);
                    } else {
                        time_since_bvh_update = 1f32 + MIN_BHV_UPDATE_TIME;
                    }
                }
                timer = rl.get_time();
            }
        }

        {
            let mut system_data: (WriteStorage<physics::Physics>, ReadStorage<utils::Position>) =
                world.system_data();
            for (phys, pos) in (&mut system_data.0, &system_data.1).join() {
                if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
                    //     let mut vec_2d = (mouse_pos - pos.0).normalized() * 10000f32
                    //         / ((mouse_pos.x - pos.0.x) * (mouse_pos.x - pos.0.x)
                    //             + (mouse_pos.y - pos.0.y) * (mouse_pos.y - pos.0.y));
                    //     let temp = vec_2d.x;
                    //     vec_2d.x = -vec_2d.y;
                    //     vec_2d.y = temp;
                    //     phys.velocity += vec_2d;
                    phys.velocity += (mouse_pos - pos.0).normalized() * 20.0;
                }
            }
        }

        let l_m_down = rl.is_mouse_button_down(MouseButton::MOUSE_RIGHT_BUTTON);

        superluminal_perf::begin_event("update_bvh");
        {
            let bvh_data: BvhData = world.system_data();
            let mut bvh_write: Write<Option<bvh::BVHTree>> = world.system_data();
            if time_since_bvh_update > MIN_BHV_UPDATE_TIME {
                *bvh_write = Some(create_bvh(bvh_data));
                // println!("{:?}", time_since_bvh_update);
                time_since_bvh_update = 0f32;
            }
        }
        superluminal_perf::end_event();

        superluminal_perf::begin_event("rendering");

        let mut d = rl.begin_drawing(&thread);
        if l_m_down || clear_once {
            d.clear_background(Color::WHITE);
            clear_once = false;
        }

        {
            let mut delta = world.read_resource::<Delta>();
            let size = world.read_resource::<[i32; 2]>();
            d.draw_rectangle(
                0,
                0,
                size[0] as i32,
                size[1] as i32,
                Color::new(
                    0,
                    0,
                    0,
                    (5f32 * delta.0 * (entity_count as f32) / 10f32) as u8,
                ),
            );
        }

        d.draw_rectangle(0, 0, 100, 50, Color::new(0, 0, 0, 255));
        {
            let mut system_data: RenderingData = world.system_data();
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
                        renderer::Renderer::CircleRenderer { radius: _, color } => {
                            *color = Color::color_from_hsv(
                                p.velocity.angle_to(Vector2::one()) * 2f32
                                    / std::f32::consts::PI.to_radians(),
                                1.0,
                                1.0,
                            );
                        }
                        renderer::Renderer::RectangeRenderer { size: _, color } => {
                            *color = Color::color_from_hsv(
                                p.velocity.angle_to(Vector2::one()) * 2f32
                                    / std::f32::consts::PI.to_radians(),
                                1.0,
                                1.0,
                            );
                        }
                    }
                }
                r.render(&mut d, pos);
                if l_m_down {
                    if let Some(c) = col {
                        let bb = c.get_bounding_box(&pos.0);
                        let bb_size = bb[1] - bb[0];
                        d.draw_rectangle_lines(
                            bb[0].x as i32,
                            bb[0].y as i32,
                            bb_size.x as i32,
                            bb_size.y as i32,
                            Color::new(0, 255, 0, 100),
                        )
                    }
                }
                // d.draw_circle_v(p.position, 10f32, Color::new(255, 0, 255, 0));
            }
        }

        d.draw_fps(0, 0);
        d.draw_text(
            format!("{:?}", entity_count).as_str(),
            0,
            20,
            20,
            if time_since_bvh_update < f32::EPSILON {
                Color::RED
            } else {
                Color::WHITE
            },
        );
        superluminal_perf::end_event();
    }
}

fn create_bvh(entities: BvhData) -> bvh::BVHTree {
    let mut data = Vec::new();

    for entity in (&entities.0, &entities.1, &entities.2).join() {
        let (ent, pos, col) = entity;
        let id = ent.id();
        // let mut hs = HashSet::new();
        // hs.insert(0);
        // hs.insert(0);
        data.push((col, pos.0, col.get_bounding_box(&pos.0), id, HS1.clone()));
    }

    bvh::BVHTree::new(data)
}
