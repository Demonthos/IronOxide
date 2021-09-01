use crate::collider;
use raylib::core::math::Vector2;
use std::collections::HashSet;

fn split_at_mid<'a>(
    mut v: Vec<(
        &'a collider::Collider,
        raylib::prelude::Vector2,
        [Vector2; 2],
        u32,
        HashSet<i8>,
    )>,
    x_axis: bool,
) -> (
    Vec<(
        &'a collider::Collider,
        Vector2,
        [Vector2; 2],
        u32,
        HashSet<i8>,
    )>,
    Vec<(
        &'a collider::Collider,
        Vector2,
        [Vector2; 2],
        u32,
        HashSet<i8>,
    )>,
) {
    // let mut v_clone = v.clone();
    let result: (
        &mut [(&collider::Collider, Vector2, [Vector2; 2], u32, HashSet<i8>)],
        &mut (&collider::Collider, Vector2, [Vector2; 2], u32, HashSet<i8>),
        &mut [(&collider::Collider, Vector2, [Vector2; 2], u32, HashSet<i8>)],
    );
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
    assert!(start.len() + end.len() == v.len());
    // println!("{:?}", v.len());
    // println!("{:?}", start.len());
    // println!("{:?}", end.len());
    (start, end)
}

enum Node {
    Branch([Vector2; 2], [Box<Node>; 2]),
    Fruit([Vector2; 2], u32, HashSet<i8>),
}

impl Node {
    fn new(mut data: Vec<(&collider::Collider, Vector2, [Vector2; 2], u32, HashSet<i8>)>) -> Node {
        if data.len() <= 1 {
            let owned = data.remove(0);
            Node::Fruit(owned.2, owned.3, owned.4)
        } else {
            // let half_size = data.len() / 2usize;
            let mut total_bb = data[0].2;
            for (_, _, bb, _, _) in &data {
                total_bb = collider::get_aabb_union(&total_bb, &bb);
            }
            let (first_half, second_half) = split_at_mid(
                data,
                (total_bb[1].x - total_bb[0].x) > (total_bb[1].y - total_bb[0].y),
            );
            let node1 = Node::new(first_half);
            let node2 = Node::new(second_half);
            Node::Branch(total_bb, [Box::new(node1), Box::new(node2)])
        }
    }

    fn get_children(&self) -> Vec<&u32> {
        let mut sum_vec = Vec::new();
        match self {
            Node::Branch(_, children) => {
                for c in children {
                    sum_vec.append(&mut c.get_children());
                }
            }
            Node::Fruit(_, other_data, _) => {
                sum_vec.push(other_data);
            }
        }
        sum_vec
    }

    fn query_point(&self, p: Vector2, layers_option: Option<&HashSet<i8>>) -> Vec<&u32> {
        let mut result = Vec::new();
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
                if bb[0].x < p.x && bb[1].x > p.x && bb[0].y < p.y && bb[1].y > p.y {
                    for child in children {
                        result.append(&mut child.query_point(p, layers_option));
                    }
                }
            }
            Node::Fruit(bb, other_data, l) => {
                let mut contains_layer = false;
                if let Some(layers) = layers_option {
                    for layer in l {
                        if layers.contains(&layer) {
                            contains_layer = true;
                            break;
                        }
                    }
                } else {
                    contains_layer = true;
                }
                if contains_layer {
                    if bb[0].x < p.x && bb[1].x > p.x && bb[0].y < p.y && bb[1].y > p.y {
                        result.push(other_data);
                    }
                }
            }
        }
        result
    }

    fn query_rect(&self, r: [Vector2; 2], layers_option: Option<&HashSet<i8>>) -> Vec<&u32> {
        let mut result = Vec::new();
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
                if collider::is_aabb_colliding(bb, &r) {
                    for child in children {
                        result.append(&mut child.query_rect(r, layers_option));
                    }
                }
            }
            Node::Fruit(bb, other_data, l) => {
                let mut contains_layer = false;
                if let Some(layers) = layers_option {
                    for layer in l {
                        if layers.contains(&layer) {
                            contains_layer = true;
                            break;
                        }
                    }
                } else {
                    contains_layer = true;
                }
                if contains_layer {
                    if collider::is_aabb_colliding(bb, &r) {
                        result.push(other_data);
                    }
                }
            }
        }
        result
    }

    fn update(&mut self, old: ([Vector2; 2], u32), new: ([Vector2; 2], u32)) -> bool {
        match self {
            Node::Branch(bb, children) => {
                if collider::is_aabb_inside(bb, &old.0) {
                    *bb = collider::get_aabb_union(bb, &new.0);
                    for c in children {
                        if c.update(old, new) {
                            return true;
                        }
                    }
                }
            }
            Node::Fruit(bb, id, _) => {
                if *id == old.1 {
                    *bb = new.0;
                    return true;
                }
            }
        }
        false
    }
}

pub struct BVHTree {
    root_node: Node,
}

impl BVHTree {
    pub fn new(
        data: Vec<(&collider::Collider, Vector2, [Vector2; 2], u32, HashSet<i8>)>,
    ) -> BVHTree {
        BVHTree {
            root_node: Node::new(data),
        }
    }

    pub fn get_all(&self) -> Vec<&u32> {
        self.root_node.get_children()
    }

    pub fn query_point(&self, p: Vector2, layers_option: Option<&HashSet<i8>>) -> Vec<&u32> {
        self.root_node.query_point(p, layers_option)
    }

    pub fn query_rect(&self, r: [Vector2; 2], layers_option: Option<&HashSet<i8>>) -> Vec<&u32> {
        self.root_node.query_rect(r, layers_option)
    }

    pub fn update(&mut self, old: ([Vector2; 2], u32), new: ([Vector2; 2], u32)) {
        self.root_node.update(old, new);
    }
}
