use crate::collider;
use raylib::core::math::Vector2;

fn split_at_mid<'a, T: std::clone::Clone>(
    mut v: Vec<(&'a collider::Collider, raylib::prelude::Vector2, T)>,
    x_axis: bool,
) -> (
    Vec<(&'a collider::Collider, Vector2, T)>,
    Vec<(&'a collider::Collider, Vector2, T)>,
) {
    // let mut v_clone = v.clone();
    let result: (
        &mut [(&collider::Collider, Vector2, T)],
        &mut (&collider::Collider, Vector2, T),
        &mut [(&collider::Collider, Vector2, T)],
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

enum Node<'a, T> {
    Branch([Vector2; 2], [Box<Node<'a, T>>; 2]),
    Fruit([Vector2; 2], &'a collider::Collider, T),
}

impl<T: std::clone::Clone> Node<'_, T> {
    fn new(mut data: Vec<(&collider::Collider, Vector2, T)>) -> Node<T> {
        if data.len() <= 1 {
            let owned = data.remove(0);
            return Node::Fruit(owned.0.get_bounding_box(&owned.1), owned.0, owned.2);
        } else {
            // let half_size = data.len() / 2usize;
            let mut total_bb = data[0].0.get_bounding_box(&data[0].1);
            for (collider, position, _) in &data {
                total_bb =
                    collider::get_aabb_union(&total_bb, &collider.get_bounding_box(&position));
            }
            let (first_half, second_half) = split_at_mid(
                data,
                (total_bb[1].x - total_bb[0].x) > (total_bb[1].y - total_bb[0].y),
            );
            let node1 = Node::new(first_half);
            let node2 = Node::new(second_half);
            return Node::Branch(total_bb, [Box::new(node1), Box::new(node2)]);
        }
    }

    fn get_children(&self) -> Vec<(&collider::Collider, &T)> {
        let mut sum_vec = Vec::new();
        match self {
            Node::Branch(_, children) => {
                for c in children {
                    sum_vec.append(&mut c.get_children());
                }
            }
            Node::Fruit(_, collider, other_data) => {
                sum_vec.push((collider, other_data));
            }
        }
        return sum_vec;
    }

    fn query_point(&self, p: Vector2) -> Vec<(&collider::Collider, &T)> {
        let mut result = Vec::new();
        match self {
            Node::Branch(bb, children) => {
                if bb[0].x < p.x && bb[1].x > p.x && bb[0].y < p.y && bb[1].y > p.y {
                    for child in children {
                        result.append(&mut child.query_point(p));
                    }
                }
            }
            Node::Fruit(bb, collider, other_data) => {
                if bb[0].x < p.x && bb[1].x > p.x && bb[0].y < p.y && bb[1].y > p.y {
                    result.push((collider, other_data));
                }
            }
        }
        result
    }

    fn query_rect(&self, r: [Vector2; 2]) -> Vec<(&collider::Collider, &T)> {
        let mut result = Vec::new();
        match self {
            Node::Branch(bb, children) => {
                if collider::is_aabb_colliding(bb, &r) {
                    for child in children {
                        result.append(&mut child.query_rect(r));
                    }
                }
            }
            Node::Fruit(bb, collider, other_data) => {
                if collider::is_aabb_colliding(bb, &r) {
                    result.push((collider, other_data));
                }
            }
        }
        result
    }
}

pub struct BVHTree<'a, T> {
    root_node: Node<'a, T>,
}

impl<T: std::clone::Clone> BVHTree<'_, T> {
    pub fn new(data: Vec<(&collider::Collider, Vector2, T)>) -> BVHTree<T> {
        BVHTree {
            root_node: Node::new(data),
        }
    }

    pub fn get_all(&self) -> Vec<(&collider::Collider, &T)> {
        self.root_node.get_children()
    }

    pub fn query_point(&self, p: Vector2) -> Vec<(&collider::Collider, &T)> {
        self.root_node.query_point(p)
    }

    pub fn query_rect(&self, r: [Vector2; 2]) -> Vec<(&collider::Collider, &T)> {
        self.root_node.query_rect(r)
    }
}
