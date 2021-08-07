/// Represents a node in the quadtree.
struct QuadNode {
    /// Points to the first child if this node is a branch or the first
    /// element if this node is a leaf.
    pub first_child: u32,

    /// Stores the number of elements in the leaf or -1 if it this node is
    /// not a leaf.
    pub count: i32,
}

pub struct QuadTree {
    nodes: Vec<QuadNode>,
}

struct QuadNodeData {
    pub index: u32,
    pub crect: [i32; 4],
    pub depth: u32,
}

#[derive(Default)]
struct QuadNodeList;

impl QuadNodeList {
    pub fn push_back(&mut self, nd: QuadNodeData) {
        todo!()
    }
}

impl QuadNodeList {
    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn pop_back(&mut self) -> QuadNodeData {
        todo!()
    }
}

impl QuadTree {
    fn find_leaves(&self, root: &QuadNodeData, rect: [i32; 4]) -> QuadNodeList {
        let mut leaves = QuadNodeList::default();
        let mut to_process = QuadNodeList::default();

        while to_process.len() > 0 {
            let nd = to_process.pop_back();

            // If this node is a leaf, insert it to the list.
            if self.nodes[nd.index as usize].count != -1 {
                leaves.push_back(nd);
                continue;
            }

            // Otherwise push the children that intersect the rectangle.
            let mx = nd.crect[0];
            let my = nd.crect[1];
            let hx = nd.crect[2] >> 1;
            let hy = nd.crect[3] >> 1;
            let fc = &self.nodes[nd.index as usize].first_child;
            let l = mx - hx;
            let t = my - hy;
            let r = mx + hx;
            let b = my + hy;

            if rect[1] <= my {
                if rect[0] <= mx {
                    to_process.push_back(Self::child_data(l, t, hx, hy, fc + 0, nd.depth + 1));
                } else {
                    to_process.push_back(Self::child_data(r, t, hx, hy, fc + 1, nd.depth + 1));
                }
            } else {
                if rect[0] <= mx {
                    to_process.push_back(Self::child_data(l, b, hx, hy, fc + 2, nd.depth + 1));
                } else {
                    to_process.push_back(Self::child_data(r, b, hx, hy, fc + 3, nd.depth + 1));
                }
            }
        }

        leaves
    }

    fn child_data(l: i32, t: i32, hx: i32, hy: i32, index: u32, depth: u32) -> QuadNodeData {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(std::mem::size_of::<QuadNode>(), 8);
    }
}
