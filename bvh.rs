use crate::collider;
use raylib::core::math::Vector2;
use rayon::prelude::*;
use std::collections::HashSet;
use std::mem;

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

#[derive(Debug, Clone)]
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
                total_bb = collider::get_aabb_union(&total_bb, bb);
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

    fn get_children(&self) -> Vec<u32> {
        let mut sum_vec = Vec::new();
        match self {
            Node::Branch(_, children) => {
                for c in children {
                    sum_vec.append(&mut c.get_children());
                }
            }
            Node::Fruit(_, other_data, _) => {
                sum_vec.push(*other_data);
            }
        }
        sum_vec
    }

    fn query_point(&self, p: Vector2, layers_option: Option<&HashSet<i8>>) -> Vec<u32> {
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
                        if layers.contains(layer) {
                            contains_layer = true;
                            break;
                        }
                    }
                } else {
                    contains_layer = true;
                }
                if contains_layer
                    && bb[0].x < p.x
                    && bb[1].x > p.x
                    && bb[0].y < p.y
                    && bb[1].y > p.y
                {
                    result.push(*other_data);
                }
            }
        }
        result
    }

    fn query_rect(
        &self,
        r: [Vector2; 2],
        layers_option: Option<&HashSet<i8>>,
        depth: i32,
    ) -> Vec<u32> {
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
                    // if depth > 1 {
                    for child in children {
                        result.append(&mut child.query_rect(r, layers_option, depth));
                    }
                    // } else {
                    //     result.par_extend(
                    //         children
                    //             .into_par_iter()
                    //             .map(|child| child.query_rect(r, layers_option, depth + 1))
                    //             .flatten(),
                    //     )
                    // }
                }
            }
            Node::Fruit(bb, other_data, l) => {
                let contains_layer = if let Some(layers) = layers_option {
                    l.into_iter().any(|layer| layers.contains(layer))
                } else {
                    true
                };
                if contains_layer && collider::is_aabb_colliding(bb, &r) {
                    result.push(*other_data);
                }
            }
        }
        result
    }

    // tries to make collision checking faster by recursively checking collision in parallel. Fails
    // fn query_rect_batched<'a>(
    //     &self,
    //     rects: &Vec<&(i32, [Vector2; 2], Option<&'a HashSet<i8>>)>,
    //     depth: i32,
    // ) -> HashMap<i32, Vec<u32>> {
    //     fn merge_hm(
    //         mut map1: HashMap<i32, Vec<u32>>,
    //         mut map2: HashMap<i32, Vec<u32>>,
    //     ) -> HashMap<i32, Vec<u32>> {
    //         if map1.len() < map2.len() {
    //             let temp = map1;
    //             map1 = map2;
    //             map2 = temp;
    //         }

    //         for (k1, v1) in map1.iter_mut() {
    //             if map2.contains_key(k1) {
    //                 v1.append(&mut map2.remove(k1).unwrap());
    //             }
    //         }

    //         for (k2, v2) in map2.drain() {
    //             if !map1.contains_key(&k2) {
    //                 map1.insert(k2, v2);
    //             }
    //         }

    //         // let k: HashSet<_> = map1.keys().cloned().collect();
    //         // map1.extend(map2.drain().filter(|(k2, _)| !k.contains(&k2)));
    //         // map1.par_extend(map2.par_drain().filter(|(k2, _)| !k.contains(&k2)));
    //         // map1.par_extend(map2.par_drain().filter(|(k2, _)| !map1.contains_key(&k2)));

    //         // map1.par_extend(map2.into_par_iter().filter(|(e, _)| !map1.contains_key(e)));
    //         map1
    //     }

    //     let mut results: HashMap<i32, Vec<u32>> = HashMap::new();
    //     let iter_rects = rects.into_par_iter();
    //     match self {
    //         Node::Branch(bb, children) => {
    //             // if let Some(layers) = layers_option {
    //             //     let mut contains_layer = false;
    //             //     for layer in l {
    //             //         if layers.contains(&layer) {
    //             //             contains_layer = true;
    //             //             break;
    //             //         }
    //             //     }
    //             //     if !contains_layer {
    //             //         return result;
    //             //     }
    //             // }
    //             let colliding = iter_rects
    //                 .filter_map(|rect| {
    //                     if collider::is_aabb_colliding(bb, &rect.1) {
    //                         Some(*rect)
    //                     } else {
    //                         None
    //                     }
    //                 })
    //                 .collect::<Vec<_>>();
    //             results = merge_hm(
    //                 results,
    //                 children
    //                     .par_iter()
    //                     .map(|child| {
    //                         child
    //                             .read()
    //                             .unwrap()
    //                             .query_rect_batched(&colliding, depth + 1)
    //                     })
    //                     .reduce(|| HashMap::new(), |hm1, hm2| merge_hm(hm1, hm2)),
    //             );
    //             // results = merge_hm(
    //             //     results,
    //             //     children
    //             //         .iter()
    //             //         .map(|child| child.read().unwrap().query_rect_batched(&colliding))
    //             //         .reduce(|hm1, hm2| merge_hm(hm1, hm2)).unwrap(),
    //             // );
    //         }
    //         Node::Fruit(bb, other_data, l) => {
    //             results = merge_hm(
    //                 results,
    //                 iter_rects
    //                     .map(|rect| {
    //                         let rect_bb = rect.1;
    //                         let mut result = Vec::new();
    //                         let mut contains_layer = false;
    //                         if let Some(layers) = rect.2 {
    //                             for layer in l {
    //                                 if layers.contains(layer) {
    //                                     contains_layer = true;
    //                                     break;
    //                                 }
    //                             }
    //                         } else {
    //                             contains_layer = true;
    //                         }
    //                         if contains_layer && collider::is_aabb_colliding(bb, &rect_bb) {
    //                             result.push(*other_data);
    //                         }
    //                         (rect.0, result)
    //                     })
    //                     .collect(),
    //             );
    //         }
    //     }
    //     results
    // }

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

    fn insert(
        &mut self,
        new: &(&collider::Collider, Vector2, [Vector2; 2], u32, HashSet<i8>),
    ) -> Option<(f32, &mut Self)> {
        let other_bb = new.2;
        let mut result: Vec<(f32, &mut Self)> = Vec::new();
        return match self {
            Node::Branch(bb, children) => {
                if collider::is_aabb_colliding(bb, &other_bb) {
                    children
                        .iter_mut()
                        .map(|child| child.insert(&new))
                        .max_by(|x, y| {
                            if let Some((n1, _)) = x {
                                if let Some((n2, _)) = y {
                                    n1.partial_cmp(n2).unwrap()
                                } else {
                                    std::cmp::Ordering::Greater
                                }
                            } else {
                                std::cmp::Ordering::Less
                            }
                        })
                        .unwrap()
                } else {
                    None
                }
            }
            Node::Fruit(bb, _, _) => {
                if collider::is_aabb_colliding(bb, &other_bb) {
                    Some(((bb[1].x - other_bb[0].x) * (bb[1].y - other_bb[0].y), self))
                } else {
                    None
                }
            }
        };
    }
}

/// This handles broad phase optimization of collisions.
/// It is a bounding volume hiarchy constructed top-down with 2 subdivisions.
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

    pub fn get_all(&self) -> Vec<u32> {
        self.root_node.get_children()
    }

    pub fn query_point(&self, p: Vector2, layers_option: Option<&HashSet<i8>>) -> Vec<u32> {
        self.root_node.query_point(p, layers_option)
    }

    pub fn query_rect(&self, r: [Vector2; 2], layers_option: Option<&HashSet<i8>>) -> Vec<u32> {
        self.root_node.query_rect(r, layers_option, 0)
    }

    pub fn update(&mut self, old: ([Vector2; 2], u32), new: ([Vector2; 2], u32)) {
        self.root_node.update(old, new);
    }

    pub fn insert(&mut self, new: &(&collider::Collider, Vector2, [Vector2; 2], u32, HashSet<i8>)) {
        if let Some((_, n)) = self.root_node.insert(new) {
            if let Node::Fruit(bb, id, layers) = n {
                let new_fruit_bb = new.0.get_bounding_box(&new.1);
                let new_branch_bb = collider::get_aabb_union(&new_fruit_bb, bb);
                mem::replace(
                    n,
                    Node::Branch(
                        new_branch_bb,
                        [
                            Box::new(Node::Fruit(new_fruit_bb, new.3, new.4.clone())),
                            Box::new(n.clone()),
                        ],
                    ),
                );
            }
        }
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
