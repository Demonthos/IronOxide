use crate::collider;
use raylib::core::math::Vector2;
use rayon::prelude::*;
use std::collections::HashSet;

fn get_union_with_map(bb1: &collider::AABB, bb2: &collider::AABB) -> (collider::AABB, [bool; 4]) {
    let extent_map = [
        bb1.lx > bb2.lx,
        bb1.rx < bb2.rx,
        bb1.ly > bb2.ly,
        bb1.ry < bb2.ry,
    ];
    let total_bb = collider::AABB {
        lx: if extent_map[0] { bb1.lx } else { bb2.lx },
        rx: if extent_map[1] { bb1.rx } else { bb2.rx },
        ly: if extent_map[2] { bb1.ly } else { bb2.ly },
        ry: if extent_map[3] { bb1.ry } else { bb2.ry },
    };
    (total_bb, extent_map)
}

type EntityData<'a> = (
    &'a collider::Collider,
    raylib::prelude::Vector2,
    collider::AABB,
    u32,
    HashSet<i8>,
);

fn split_at_mid(mut v: Vec<EntityData>, x_axis: bool) -> (Vec<EntityData>, Vec<EntityData>) {
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
    (start, end)
}

#[derive(Debug, Clone)]
pub enum Node {
    // stores bounding box, children and a boolean array maping the bounds with the fault of the extent, not faster, so removed
    Branch(collider::AABB, [Box<Node>; 2], [bool; 4]),
    // Branch(collider::AABB, [Box<Node>; 2]),
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
                Node::Branch(ref bb, _, _) => bb,
                Node::Fruit(ref bb, _, _) => bb,
            };
            let bb2 = match node2 {
                Node::Branch(ref bb, _, _) => bb,
                Node::Fruit(ref bb, _, _) => bb,
            };
            let (total_bb, extent_map) = get_union_with_map(bb1, bb2);
            Node::Branch(total_bb, [Box::new(node1), Box::new(node2)], extent_map)
        }
    }

    fn shrink(&mut self) -> &collider::AABB {
        match self {
            Node::Branch(bb, children, _) => {
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
            Node::Branch(_, children, _) => {
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
        match self {
            Node::Branch(_, children, _) => {
                for c in children {
                    sum_vec.append(&mut c.get_children());
                }
            }
            _ => (),
        }
        sum_vec.push(self);
        sum_vec
    }

    fn query_point(&self, p: Vector2, layers_option: Option<&HashSet<i8>>) -> Option<Vec<u32>> {
        let mut result: Option<Vec<u32>> = None;
        match self {
            Node::Branch(bb, children, _) => {
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

    fn query_rect(
        &self,
        r: &collider::AABB,
        layers_option: Option<&HashSet<i8>>,
        depth: i32,
    ) -> Option<Vec<u32>> {
        match self {
            Node::Branch(_, children, extent_map) => {
                let mut result: Option<Vec<u32>> = None;
                let bb = match &*children[0] {
                    Node::Branch(bb, _, _) => bb,
                    Node::Fruit(bb, _, _) => bb,
                };
                if bb.is_colliding_with_map(&r, *extent_map) {
                    // let child_results = ;
                    if let Some(mut child_collisions) =
                        children[0].query_rect(&r, layers_option, depth + 1)
                    {
                        if let Some(ref mut result_vec) = result {
                            result_vec.append(&mut child_collisions);
                        } else {
                            result = Some(child_collisions);
                        }
                    }
                }
                let bb = match &*children[1] {
                    Node::Branch(bb, _, _) => bb,
                    Node::Fruit(bb, _, _) => bb,
                };
                if bb.is_colliding_with_map(
                    &r,
                    [
                        !extent_map[0],
                        !extent_map[1],
                        !extent_map[2],
                        !extent_map[3],
                    ],
                ) {
                    // let child_results = ;
                    if let Some(mut child_collisions) =
                        children[1].query_rect(&r, layers_option, depth + 1)
                    {
                        if let Some(ref mut result_vec) = result {
                            result_vec.append(&mut child_collisions);
                        } else {
                            result = Some(child_collisions);
                        }
                    }
                }
                result
            }
            Node::Fruit(_, other_data, l) => {
                let contains_layer = if let Some(layers) = layers_option {
                    l.iter().any(|layer| layers.contains(layer))
                } else {
                    true
                };
                if contains_layer {
                    Some(vec![*other_data])
                } else {
                    None
                }
            }
        }
    }

    fn update(&mut self, old: (&collider::AABB, u32), new: (&collider::AABB, u32)) -> bool {
        match self {
            Node::Branch(bb, children, _) => {
                if bb.is_inside(&old.0) {
                    for c in children {
                        if c.update(old, new) {
                            return true;
                        }
                    }
                    *bb = bb.get_union(&new.0);
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
            Node::Branch(bb, children, _) => {
                *bb = bb.get_union(&new_fruit_bb);
                children[0].insert(new);
            }
            Node::Fruit(bb, _, _) => {
                let (new_branch_bb, extent_map) = get_union_with_map(&new_fruit_bb, bb);
                *self = Node::Branch(
                    new_branch_bb,
                    [
                        Box::new(Node::Fruit(new_fruit_bb, new.3, new.4.clone())),
                        Box::new(self.clone()),
                    ],
                    extent_map,
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
        let bb = match &self.root_node {
            Node::Branch(bb, _, _) => bb,
            Node::Fruit(bb, _, _) => bb,
        };
        if bb.is_colliding(&r) {
            self.root_node
                .query_rect(&r, layers_option, 0)
                .unwrap_or_default()
        } else {
            Vec::new()
        }
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
    }

    pub fn shrink(&mut self) {
        self.root_node.shrink();
    }
}
