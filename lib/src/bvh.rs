use crate::collider;
use core::cmp::min;
use raylib::core::math::Vector2;
use rayon::prelude::*;
use std::collections::HashSet;

type EntityData<'a> = (
    &'a collider::Collider,
    raylib::prelude::Vector2,
    collider::AABB,
    u32,
    HashSet<i8>,
);

fn split_at_mid(mut v: Vec<EntityData>, x_axis: bool) -> (Vec<EntityData>, Vec<EntityData>) {
    // let mut v_clone = v.clone();
    let result: (&mut [EntityData], &mut EntityData, &mut [EntityData]);
    let half_size = (v.len() / 2usize) - 1;

    if x_axis {
        result = v.select_nth_unstable_by(half_size, |vec1, vec2| {
            vec1.1.x.partial_cmp(&vec2.1.x).unwrap()
        });
    } else {
        result = v.select_nth_unstable_by(half_size, |vec1, vec2| {
            vec1.1.y.partial_cmp(&vec2.1.y).unwrap()
        });
    }
    let mut start = result.0.to_vec();
    start.push(result.1.clone());
    let end = result.2.to_vec();
    assert_eq!(start.len() + end.len(), v.len());
    // println!("{:?}", v.len());
    // println!("{:?}", start.len());
    // println!("{:?}", end.len());
    (start, end)
}

#[derive(Debug, Clone)]
pub enum Node {
    Branch(collider::AABB, [Box<Node>; 2]),
    Fruit(collider::AABB, u32, HashSet<i8>),
}

impl Node {
    // 1/2 of bounding box data is redundant!
    // make more cache efficient
    // add collision cache?
    fn new(mut data: Vec<EntityData>) -> Node {
        if data.len() <= 1 {
            let owned = data.remove(0);
            Node::Fruit(owned.2, owned.3, owned.4)
        } else {
            // let half_size = data.len() / 2usize;
            let mut total_bb = data[0].2.clone();
            for e in &data {
                total_bb = total_bb.with_point(&e.1);
            }
            let (first_half, second_half) = split_at_mid(
                data,
                (total_bb.rx - total_bb.lx) > (total_bb.ry - total_bb.ly),
            );
            let node1 = Node::new(first_half);
            let node2 = Node::new(second_half);
            let bb1 = match node1 {
                Node::Branch(ref bb, _) => bb,
                Node::Fruit(ref bb, _, _) => bb,
            };
            let bb2 = match node2 {
                Node::Branch(ref bb, _) => bb,
                Node::Fruit(ref bb, _, _) => bb,
            };
            let total_bb = bb1.get_union(bb2);
            Node::Branch(total_bb, [Box::new(node1), Box::new(node2)])
        }
    }

    fn shrink(&mut self) -> &collider::AABB {
        match self {
            Node::Branch(bb, children) => {
                let mut children_iter = children.iter_mut();
                if let Some(first) = children_iter.next() {
                    *bb = first.shrink().clone();
                    for c in children_iter {
                        *bb = bb.get_union(c.shrink());
                    }
                };
                bb
            }
            Node::Fruit(bb, _, _) => bb,
        }
    }

    fn get_children_id(&self) -> Vec<u32> {
        let mut sum_vec = Vec::new();
        match self {
            Node::Branch(_, children) => {
                for c in children {
                    sum_vec.append(&mut c.get_children_id());
                }
            }
            Node::Fruit(_, other_data, _) => {
                sum_vec.push(*other_data);
            }
        }
        sum_vec
    }

    fn get_children(&self) -> Vec<&Node> {
        let mut sum_vec = Vec::new();
        if let Node::Branch(_, children) = self {
            for c in children {
                sum_vec.append(&mut c.get_children());
            }
        }
        sum_vec.push(self);
        sum_vec
    }

    fn query_point(&self, p: Vector2, layers_option: Option<&HashSet<i8>>) -> Option<Vec<u32>> {
        let mut result: Option<Vec<u32>> = None;
        match self {
            Node::Branch(bb, children) => {
                // if let Some(layers) = layers_option {
                //     let mut contains_layer = false;
                //     for layer in l {
                //         if layers.contains(&layer) {
                //             contains_layer = true;
                //             break;
                //         }
                //     }
                //     if !contains_layer {
                //         return result;
                //     }
                // }
                if bb.lx < p.x && bb.rx > p.x && bb.ly < p.y && bb.ry > p.y {
                    for child in children {
                        let child_results = child.query_point(p, layers_option);
                        if let Some(mut child_collisions) = child_results {
                            if let Some(ref mut result_vec) = result {
                                result_vec.append(&mut child_collisions);
                            } else {
                                result = Some(child_collisions);
                            }
                        }
                    }
                }
            }
            Node::Fruit(bb, other_data, l) => {
                let mut contains_layer = false;
                if let Some(layers) = layers_option {
                    for layer in l {
                        if layers.contains(layer) {
                            contains_layer = true;
                            break;
                        }
                    }
                } else {
                    contains_layer = true;
                }
                if contains_layer && bb.lx < p.x && bb.rx > p.x && bb.ly < p.y && bb.ry > p.y {
                    if let Some(ref mut result_vec) = result {
                        result_vec.push(*other_data);
                    } else {
                        result = Some(vec![*other_data]);
                    }
                }
            }
        }
        result
    }

    fn debug_query_point(
        &self,
        p: Vector2,
        layers_option: Option<&HashSet<i8>>,
        depth: i32,
    ) -> (Option<Vec<u32>>, Vec<(&Node, i32)>) {
        match self {
            Node::Branch(bb, children) => {
                // if let Some(layers) = layers_option {
                //     let mut contains_layer = false;
                //     for layer in l {
                //         if layers.contains(&layer) {
                //             contains_layer = true;
                //             break;
                //         }
                //     }
                //     if !contains_layer {
                //         return None;
                //     }
                // }
                let mut result: (Option<Vec<u32>>, Vec<(&Node, i32)>) = (None, Vec::new());
                // if depth > 1 {
                if bb.lx < p.x && bb.rx > p.x && bb.ly < p.y && bb.ry > p.y {
                    for child in children {
                        // let child_results = ;
                        let mut child_data = child.debug_query_point(p, layers_option, depth + 1);
                        result.1.append(&mut child_data.1);
                        if let Some(mut child_collisions) = child_data.0 {
                            if let Some(ref mut result_vec) = result.0 {
                                result_vec.append(&mut child_collisions);
                            } else {
                                result.0 = Some(child_collisions);
                            }
                        }
                    }
                    // } else {
                    //     result.par_extend(
                    //         children
                    //             .into_par_iter()
                    //             .map(|child| child.query_rect(r, layers_option, depth + 1))
                    //             .flatten(),
                    //     )
                    // }
                    result
                } else {
                    (None, vec![(self, depth)])
                }
            }
            Node::Fruit(bb, other_data, l) => {
                let contains_layer = if let Some(layers) = layers_option {
                    l.iter().any(|layer| layers.contains(layer))
                } else {
                    true
                };
                if contains_layer && bb.lx < p.x && bb.rx > p.x && bb.ly < p.y && bb.ry > p.y {
                    (Some(vec![*other_data]), vec![(self, depth)])
                } else {
                    (None, vec![(self, depth)])
                }
            }
        }
    }

    fn query_rect(
        &self,
        r: &collider::AABB,
        layers_option: Option<&HashSet<i8>>,
        depth: i32,
    ) -> Option<Vec<u32>> {
        match self {
            Node::Branch(bb, children) => {
                // if let Some(layers) = layers_option {
                //     let mut contains_layer = false;
                //     for layer in l {
                //         if layers.contains(&layer) {
                //             contains_layer = true;
                //             break;
                //         }
                //     }
                //     if !contains_layer {
                //         return None;
                //     }
                // }
                let mut result: Option<Vec<u32>> = None;
                // if depth > 1 {
                if bb.is_colliding(r) {
                    for child in children {
                        // let child_results = ;
                        if let Some(mut child_collisions) =
                            child.query_rect(r, layers_option, depth + 1)
                        {
                            if let Some(ref mut result_vec) = result {
                                result_vec.append(&mut child_collisions);
                            } else {
                                result = Some(child_collisions);
                            }
                        }
                    }
                    // } else {
                    //     result.par_extend(
                    //         children
                    //             .into_par_iter()
                    //             .map(|child| child.query_rect(r, layers_option, depth + 1))
                    //             .flatten(),
                    //     )
                    // }
                    result
                } else {
                    None
                }
            }
            Node::Fruit(bb, other_data, l) => {
                let contains_layer = if let Some(layers) = layers_option {
                    l.iter().any(|layer| layers.contains(layer))
                } else {
                    true
                };
                if contains_layer && bb.is_colliding(r) {
                    Some(vec![*other_data])
                } else {
                    None
                }
            }
        }
    }

    fn debug_query_rect(
        &self,
        r: &collider::AABB,
        layers_option: Option<&HashSet<i8>>,
        depth: i32,
    ) -> (Option<Vec<u32>>, Vec<(&Node, i32)>) {
        match self {
            Node::Branch(bb, children) => {
                // if let Some(layers) = layers_option {
                //     let mut contains_layer = false;
                //     for layer in l {
                //         if layers.contains(&layer) {
                //             contains_layer = true;
                //             break;
                //         }
                //     }
                //     if !contains_layer {
                //         return None;
                //     }
                // }
                let mut result: (Option<Vec<u32>>, Vec<(&Node, i32)>) = (None, Vec::new());
                // if depth > 1 {
                if bb.is_colliding(r) {
                    for child in children {
                        // let child_results = ;
                        let mut child_data = child.debug_query_rect(r, layers_option, depth + 1);
                        result.1.append(&mut child_data.1);
                        if let Some(mut child_collisions) = child_data.0 {
                            if let Some(ref mut result_vec) = result.0 {
                                result_vec.append(&mut child_collisions);
                            } else {
                                result.0 = Some(child_collisions);
                            }
                        }
                    }
                    // } else {
                    //     result.par_extend(
                    //         children
                    //             .into_par_iter()
                    //             .map(|child| child.query_rect(r, layers_option, depth + 1))
                    //             .flatten(),
                    //     )
                    // }
                    result
                } else {
                    (None, vec![(self, depth)])
                }
            }
            Node::Fruit(bb, other_data, l) => {
                let contains_layer = if let Some(layers) = layers_option {
                    l.iter().any(|layer| layers.contains(layer))
                } else {
                    true
                };
                if contains_layer && bb.is_colliding(r) {
                    (Some(vec![*other_data]), vec![(self, depth)])
                } else {
                    (None, vec![(self, depth)])
                }
            }
        }
    }

    fn update(&mut self, old: (&collider::AABB, u32), new: (&collider::AABB, u32)) -> bool {
        match self {
            Node::Branch(bb, children) => {
                if bb.is_inside(old.0) {
                    for c in children {
                        if c.update(old, new) {
                            return true;
                        }
                    }
                    *bb = bb.get_union(new.0);
                }
            }
            Node::Fruit(bb, id, _) => {
                if *id == old.1 {
                    *bb = new.0.clone();
                    return true;
                }
            }
        }
        false
    }

    fn insert(
        &mut self,
        new: &(
            &collider::Collider,
            Vector2,
            collider::AABB,
            u32,
            HashSet<i8>,
        ),
    ) {
        let new_fruit_bb = new.0.get_bounding_box(&new.1);
        match self {
            Node::Branch(bb, children) => {
                *bb = bb.get_union(&new_fruit_bb);
                let mut best_dist = match &*children[0] {
                    Node::Branch(bb2, _) => bb2.get_dist(&new.2),
                    Node::Fruit(bb2, _, _) => bb2.get_dist(&new.2),
                };
                let (ref mut first, rest) = children.split_at_mut(1);
                let mut best = &mut *first[0];
                for child in rest {
                    let new_dist = match &**child {
                        Node::Branch(bb2, _) => bb2.get_dist(&new.2),
                        Node::Fruit(bb2, _, _) => bb2.get_dist(&new.2),
                    };
                    if new_dist < best_dist {
                        best_dist = new_dist;
                        best = &mut **child;
                    }
                }
                best.insert(new);
            }
            Node::Fruit(bb, _, _) => {
                // let (new_branch_bb, extent_map) = get_union_with_map(&new_fruit_bb, bb);
                let new_branch_bb = new_fruit_bb.get_union(bb);
                *self = Node::Branch(
                    new_branch_bb,
                    [
                        Box::new(Node::Fruit(new_fruit_bb, new.3, new.4.clone())),
                        Box::new(self.clone()),
                    ],
                    // extent_map,
                );
            }
        }
    }
}

/// This handles broad phase optimization of collisions.
/// It is a bounding volume hierarchy constructed top-down with 2 subdivisions.
pub struct BVHTree {
    root_node: Node,
}

impl BVHTree {
    pub fn new(
        data: Vec<(
            &collider::Collider,
            Vector2,
            collider::AABB,
            u32,
            HashSet<i8>,
        )>,
    ) -> BVHTree {
        BVHTree {
            root_node: Node::new(data),
        }
    }

    pub fn get_children_id(&self) -> Vec<u32> {
        self.root_node.get_children_id()
    }

    pub fn get_children(&self) -> Vec<&Node> {
        self.root_node.get_children()
    }

    pub fn query_point(&self, p: Vector2, layers_option: Option<&HashSet<i8>>) -> Vec<u32> {
        self.root_node
            .query_point(p, layers_option)
            .unwrap_or_default()
    }

    pub fn query_rect(&self, r: collider::AABB, layers_option: Option<&HashSet<i8>>) -> Vec<u32> {
        self.root_node
            .query_rect(&r, layers_option, 0)
            .unwrap_or_default()
    }

    pub fn debug_query_rect(
        &self,
        r: &collider::AABB,
        layers_option: Option<&HashSet<i8>>,
    ) -> (Option<Vec<u32>>, Vec<(&Node, i32)>) {
        self.root_node.debug_query_rect(r, layers_option, 0)
    }

    pub fn debug_query_point(
        &self,
        p: Vector2,
        layers_option: Option<&HashSet<i8>>,
    ) -> (Option<Vec<u32>>, Vec<(&Node, i32)>) {
        self.root_node.debug_query_point(p, layers_option, 0)
    }

    pub fn update(&mut self, old: (&collider::AABB, u32), new: (&collider::AABB, u32)) {
        self.root_node.update(old, new);
    }

    pub fn insert(
        &mut self,
        new: &(
            &collider::Collider,
            Vector2,
            collider::AABB,
            u32,
            HashSet<i8>,
        ),
    ) {
        self.root_node.insert(new);
        // println!("{:#?}", result);
    }

    pub fn shrink(&mut self) {
        self.root_node.shrink();
    }

    // pub fn query_rect_batched<'a>(
    //     &self,
    //     rects: &Vec<(i32, [Vector2; 2], Option<&'a HashSet<i8>>)>,
    // ) -> HashMap<i32, Vec<u32>> {
    //     superluminal_perf::begin_event("batched");
    //     let r = self
    //         .root_node
    //         .query_rect_batched(&(rects.into_iter().collect()), 0);
    //     superluminal_perf::end_event();
    //     r
    // }
}
