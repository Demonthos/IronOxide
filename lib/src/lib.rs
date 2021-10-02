#[macro_use]
extern crate lazy_static;
pub use rand;
pub use raylib::prelude::*;
pub use rayon::prelude::*;
pub use specs::Dispatcher;
pub use specs::DispatcherBuilder;
pub use specs::{
    Builder, Entities, Join, ParJoin, Read, ReadStorage, System, World, WorldExt, Write,
    WriteStorage,
};
use std::collections::HashSet;

pub mod bvh;
pub mod collider;
pub mod physics;
pub mod renderer;
pub mod utils;
// mod tests;

// const COLLISION_FRICTION: f32 = 0.998f32;
const COLLISION_FRICTION: f32 = 1f32;
// const FRICTION: f32 = 0.998f32;
const FRICTION: f32 = 1f32;
// const GRAVITY: f32 = 1f32;
const GRAVITY: f32 = 0f32;
// const MIN_BHV_UPDATE_TIME: f32 = 100f32;
const MIN_BHV_UPDATE_TIME: f32 = 0.25f32;
const WINDOW_SIZE: [i32; 2] = [1400, 1000];
lazy_static! {
    static ref HS1: HashSet<i8> = vec![0].into_iter().collect();
    static ref HS2: HashSet<i8> = vec![1].into_iter().collect();
}

pub type RenderingData<'a> = (
    WriteStorage<'a, renderer::Renderer>,
    ReadStorage<'a, utils::Position>,
    ReadStorage<'a, physics::Physics>,
    WriteStorage<'a, collider::Collider>,
);

pub type BvhData<'a> = (
    Entities<'a>,
    ReadStorage<'a, utils::Position>,
    ReadStorage<'a, collider::Collider>,
);

struct UpdatePhysics;

impl<'a> System<'a> for UpdatePhysics {
    type SystemData = (
        Write<'a, Option<bvh::BVHTree>>,
        Entities<'a>,
        ReadStorage<'a, collider::Collider>,
        Read<'a, utils::Delta>,
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
                // if old_pos.distance_to(pos.0) < 0.0000001 {
                //     println!("{:#?}, {:#?}, {:#?}, {}", phys, old_pos, pos.0, delta.0);
                // }
                if let Some(col) = col_m {
                    bvh.update(
                        (&col.get_bounding_box(&old_pos), ent.id() as u32),
                        (&col.get_bounding_box(&pos.0), ent.id() as u32),
                    );
                }
            }
        }
    }
}

struct ShrinkBvh;

impl<'a> System<'a> for ShrinkBvh {
    type SystemData = Write<'a, Option<bvh::BVHTree>>;

    fn run(&mut self, mut bvh_tree: Self::SystemData) {
        // make this parrelel
        if let Some(ref mut bvh) = *bvh_tree {
            bvh.shrink();
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
        Entities<'a>,
        WriteStorage<'a, utils::Collisions>,
    );

    fn run(&mut self, mut data: Self::SystemData) {
        let bvh_tree = data.0;
        let entity_data = (
            &mut data.1,
            &data.2,
            (&mut data.3).maybe(),
            &data.4,
            &mut data.5,
        )
            .join()
            .collect::<Vec<_>>();

        if let Some(ref bvh) = *bvh_tree {
            // costly
            // let old_data: Vec<_> = entity_data
            //     .iter()
            //     .map(|e| Some((e.0 .0, e.2.as_deref().cloned(), e.1.clone())))
            //     .collect();

            let mut old_data = Vec::new();

            for e in &entity_data {
                let id = e.3.id() as usize;
                old_data.resize(id + 1, None);
                old_data[id] = Some((e.0 .0, e.2.as_deref().cloned(), e.1.clone()));
            }

            // let old_data: Vec<_> = entity_data.iter()
            //     .map(|e| (e.0 .0, e.2.as_deref().cloned(), e.1.clone()))
            //     .collect();

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

            entity_data.into_par_iter().for_each(|ref mut p| {
                let first_id = p.3.id();
                let old = old_data[first_id as usize].as_ref().unwrap().0;
                let collisions: Vec<_> = bvh
                    .query_rect(&p.1.get_bounding_box(&old), &p.1.collision_mask)
                    .iter()
                    .filter(|id| **id != first_id)
                    .copied()
                    .collect();
                for p2_id in &collisions {
                    // if old_data[*p2_id as usize].is_none() {
                    //     println!("{:#?}", *p2_id);
                    // }
                    let p2 = old_data[*p2_id as usize].as_ref().unwrap();
                    let overlap_vec = p.1.get_collision(&old, &p2.0, &p2.2);
                    if let Some(unwraped) = overlap_vec {
                        // make sure collisions are not handled twice, but we calculate it twice
                        if p.1.physics_collider && p2.2.physics_collider {
                            if let Some(ref mut phys) = p.2 {
                                if let Some(p2_phys) = &p2.1 {
                                    phys.resolve_collision_single(
                                        &mut p.0 .0,
                                        &p2.0,
                                        p2_phys,
                                        unwraped,
                                    );
                                }
                            }
                        }
                    }
                }
                *p.4 = utils::Collisions(collisions);
            });
        }
    }
}

/// Builds the world
pub fn build<'a, 'b>() -> (RaylibHandle, RaylibThread, World, DispatcherBuilder<'a, 'b>) {
    let (rl, thread) = raylib::init()
        .resizable()
        // .transparent()
        // .undecorated()
        .size(WINDOW_SIZE[0], WINDOW_SIZE[1])
        .title("Iron Oxide Engine")
        .build();

    let _time_since_bvh_update = 0f32;
    let bvh_tree: Option<bvh::BVHTree> = None;

    let mut world = World::new();
    world.register::<utils::Position>();
    world.register::<utils::Collisions>();
    world.register::<physics::Physics>();
    world.register::<collider::Collider>();
    world.register::<renderer::Renderer>();
    world.insert(utils::Delta(0.00));
    world.insert([rl.get_screen_width(), rl.get_screen_height()]);
    world.insert(bvh_tree);
    let dispatcher = DispatcherBuilder::new()
        .with(UpdatePhysics, "update_physics", &[])
        .with(CollideBounds, "collide_bounds", &["update_physics"])
        .with(CollideEnities, "collide_entities", &["update_physics"])
        .with(ShrinkBvh, "shrink_bvh", &[]);
    (rl, thread, world, dispatcher)
}

/// Finalizes the world, run this after adding custom systems
pub fn init<'a, 'b>(
    state: (RaylibHandle, RaylibThread, World, DispatcherBuilder<'a, 'b>),
) -> (
    raylib::RaylibHandle,
    raylib::RaylibThread,
    World,
    Dispatcher<'a, 'b>,
    f32,
) {
    let dispatcher = state.3.build();

    let time_since_bvh_update = 0f32;

    (state.0, state.1, state.2, dispatcher, time_since_bvh_update)
}

/// Run this every frame
pub fn update<'a, 'b>(
    state: &mut (
        raylib::RaylibHandle,
        raylib::RaylibThread,
        World,
        Dispatcher<'a, 'b>,
        f32,
    ),
    callback: fn(&mut World, &mut raylib::prelude::RaylibDrawHandle),
) {
    let (rl, thread, world, dispatcher, time_since_bvh_update) = state;

    {
        let mut delta = world.write_resource::<utils::Delta>();
        *delta = utils::Delta(rl.get_frame_time());
        *time_since_bvh_update += delta.0;
    }

    // update screen size
    if rl.is_window_resized() {
        let mut size = world.write_resource::<[i32; 2]>();
        *size = [rl.get_screen_width(), rl.get_screen_height()]
    }

    world.maintain();
    dispatcher.dispatch(world);

    // update bvh
    {
        let bvh_data: BvhData = world.system_data();
        let mut bvh_write: Write<Option<bvh::BVHTree>> = world.system_data();
        if *time_since_bvh_update > MIN_BHV_UPDATE_TIME || bvh_write.is_none() {
            *bvh_write = create_bvh(bvh_data);
            // println!("{:?}", time_since_bvh_update);
            *time_since_bvh_update = 0f32;
        }
    }

    // draw everything
    {
        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::WHITE);

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
                let (r, pos, _phys, _col) = data;
                r.render(&mut d, pos);
            }
        }
        callback(world, &mut d);

        d.draw_fps(0, 0);
    }
}

pub fn create_bvh(entities: BvhData) -> Option<bvh::BVHTree> {
    let mut data = Vec::new();

    for entity in (&entities.0, &entities.1, &entities.2).join() {
        let (ent, pos, col) = entity;
        let id = ent.id();
        // let mut hs = HashSet::new();
        // hs.insert(0);
        // hs.insert(0);
        data.push((col, pos.0, col.get_bounding_box(&pos.0), id));
    }

    if data.len() > 0 {
        Some(bvh::BVHTree::new(data))
    } else {
        None
    }
}
