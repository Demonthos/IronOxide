use crate::collider;
use raylib::core::math::Vector2;

fn split_at_mid(
    v: &mut Vec<Vector2>,
    x_axis: bool,
) -> (&mut [Vector2], &mut Vector2, &mut [Vector2]) {
    // let mut v_clone = v.clone();
    let result: (&mut [Vector2], &mut Vector2, &mut [Vector2]);
    let half_size = v.len() / 2usize;
    if x_axis {
        result =
            v.select_nth_unstable_by(half_size, |vec1, vec2| vec1.x.partial_cmp(&vec2.x).unwrap());
    } else {
        result =
            v.select_nth_unstable_by(half_size, |vec1, vec2| vec1.y.partial_cmp(&vec2.y).unwrap());
    }
    result
}

enum Node<'a> {
    Branch([Vector2; 2], [Box<Node<'a>>; 2]),
    Fruit([Vector2; 2], &'a collider::Collider),
}

impl Node<'_> {
    fn new(colliders: Vec<&collider::Collider>, positions: Vec<Vector2>) -> Node {
        if colliders.len() <= 1 {
            return Node::Fruit(colliders[0].get_bounding_box(&positions[0]), colliders[0]);
        } else {
            let half_size = colliders.len() / 2usize;
            let node1 = Node::new(
                colliders[..half_size].to_vec(),
                positions[..half_size].to_vec(),
            );
            let aabb1: [Vector2; 2];
            match node1 {
                Node::Branch(aabb, _) => aabb1 = aabb.clone(),
                Node::Fruit(aabb, _) => aabb1 = aabb.clone(),
            }
            let node2 = Node::new(
                colliders[half_size..].to_vec(),
                positions[half_size..].to_vec(),
            );
            let aabb2: [Vector2; 2];
            match node2 {
                Node::Branch(aabb, _) => aabb2 = aabb.clone(),
                Node::Fruit(aabb, _) => aabb2 = aabb.clone(),
            }
            return Node::Branch(
                collider::get_aabb_union(&aabb1, &aabb2),
                [Box::new(node1), Box::new(node2)],
            );
        }
    }

    fn get_children(&self) -> Vec<&collider::Collider> {
        let mut sum_vec = Vec::new();
        match self {
            Node::Branch(_, children) => {
                for c in children {
                    sum_vec.append(&mut c.get_children());
                }
            }
            Node::Fruit(_, collider) => {
                sum_vec.push(collider);
            }
        }
        return sum_vec;
    }

    fn query_point(&self, p: Vector2) -> Vec<&collider::Collider> {
        let mut result = Vec::new();
        match self {
            Node::Branch(bb, children) => {
                if bb[0].x < p.x && bb[1].x > p.x && bb[0].y < p.y && bb[1].y > p.y {
                    for child in children {
                        result.append(&mut child.query_point(p));
                    }
                }
            }
            Node::Fruit(bb, collider) => {
                if bb[0].x < p.x && bb[1].x > p.x && bb[0].y < p.y && bb[1].y > p.y {
                    result.push(collider);
                }
            }
        }
        result
    }

    fn query_rect(&self, r: [Vector2; 2]) -> Vec<&collider::Collider> {
        let mut result = Vec::new();
        match self {
            Node::Branch(bb, children) => {
                if collider::is_aabb_colliding(bb, &r) {
                    for child in children {
                        result.append(&mut child.query_rect(r));
                    }
                }
            }
            Node::Fruit(bb, collider) => {
                if collider::is_aabb_colliding(bb, &r) {
                    result.push(collider);
                }
            }
        }
        result
    }
}

pub struct BVHTree<'a> {
    root_node: Node<'a>,
}

impl BVHTree<'_> {
    pub fn new(colliders: Vec<&collider::Collider>, positions: Vec<Vector2>) -> BVHTree {
        BVHTree {
            root_node: Node::new(colliders, positions),
        }
    }

    pub fn get_all(&self) -> Vec<&collider::Collider> {
        self.root_node.get_children()
    }

    pub fn query_point(&self, p: Vector2) -> Vec<&collider::Collider> {
        self.root_node.query_point(p)
    }

    pub fn query_rect(&self, r: [Vector2; 2]) -> Vec<&collider::Collider> {
        self.root_node.query_rect(r)
    }
}
