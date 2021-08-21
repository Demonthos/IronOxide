use crate::collider;
use raylib::core::math::Vector2;

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
}

pub struct BVHTree<'a> {
    root_node: Node<'a>,
}

impl BVHTree<'_> {
    fn new(colliders: Vec<&collider::Collider>, positions: Vec<Vector2>) -> BVHTree {
        BVHTree {
            root_node: Node::new(colliders, positions),
        }
    }

    fn get_all(&self) -> Vec<&collider::Collider> {
        self.root_node.get_children()
    }
}
