use bevy::prelude::Vec2;
use core::panic;
use std::vec;

/// Contains the information regarding the node itself and also the
/// index of it's children.
#[derive(Debug, Copy, Clone)]
#[readonly::make]
pub struct Node {
    /// Indices to child nodes, going clockwise from top-left.
    children: [Option<usize>; 4],
    /// Mass of the node
    pub mass: f32,
    /// Center of the region the node is representing
    center: Vec2,
    /// Center of mass of the node (equal to position if the node is a
    /// leaf node)
    pub center_of_mass: Vec2,
    /// Distance from center to the side of the square
    half_size: f32,
}

/// Stores information about the quadtree.
#[readonly::make]
pub struct QuadTree {
    /// The inner vector, storing the nodes
    vec: Vec<Node>,
    /// The bounds of area covered by the quadtree.
    /// <div class="warning">
    /// Should always be a square,
    /// otherwise operations on the tree *will* be invalid.
    /// </div>
    bounds: [Vec2; 2],
    /// The index of root node
    pub root: usize,
}

impl Node {
    // Returns the index of quadrant to which the position belongs.
    // WARNING!!! pos should be inside the bounds of this node, otherwise
    // the quadtree structure is invalid if
    // the index is then used to append stuff.
    fn get_quadrant(&self, pos: Vec2) -> usize {
        match (pos.x > self.center.x, pos.y > self.center.y) {
            (false, true) => 0,  // top-left
            (true, true) => 1,   // top-right
            (false, false) => 2, // bottom-left
            (true, false) => 3,  // bottom-right
        }
    }

    // Whether this node is a leaf node.
    fn is_leaf(&self) -> bool {
        // Leaf nodes don't have any children.
        for n in self.children {
            if n.is_some() {
                return false;
            }
        }
        return true;
    }
}

impl QuadTree {
    /// Construct a new Quadtree using center and half size, to construct a
    /// square bounding box.
    pub fn new(center: Vec2, half_size: f32) -> Self {
        let xy1 = Vec2::new(center.x - half_size, center.y - half_size);
        let xy2 = Vec2::new(center.x + half_size, center.y + half_size);
        QuadTree {
            vec: vec![Node {
                children: [None; 4],
                mass: 0.,
                center,
                center_of_mass: center,
                half_size,
            }],
            bounds: [xy1, xy2],
            root: 0,
        }
    }

    /// Returns true if the `position` is inside the bounds of this quadtree
    fn in_bounds(&mut self, position: Vec2) -> bool {
        // Out of bounds to the left.
        if position.x < self.bounds[0].x {
            return false;
        }
        // Out of bounds to the right.
        if self.bounds[1].x < position.x {
            return false;
        }
        // Out of bounds at the top
        if position.y < self.bounds[0].y {
            return false;
        }
        // Out of bounds at the bottom
        if self.bounds[1].y < position.y {
            return false;
        }

        return true;
    }

    /// Finds the leaf node that needs to be split to insert the new node and
    /// splits it using recursion.
    fn split_add_recursive(&mut self, node_idx: usize, position: Vec2, mass: f32) {
        let child_quadrant;
        let new_halfsize;
        let center;

        {
            let node = &mut self.vec[node_idx];
            // Recalculate the center of mass and mass of this node with the
            // passed arguments
            node.center_of_mass =
                (node.center_of_mass * node.mass + position * mass) / (node.mass + mass);
            node.mass += mass;
            // Get the quadrant where the position would belong and the center
            // of that quadrant
            child_quadrant = node.get_quadrant(position);
            new_halfsize = node.half_size / 2.;
            center = match child_quadrant {
                0 => Vec2::new(node.center.x - new_halfsize, node.center.y - new_halfsize),
                1 => Vec2::new(node.center.x + new_halfsize, node.center.y - new_halfsize),
                2 => Vec2::new(node.center.x + new_halfsize, node.center.y + new_halfsize),
                3 => Vec2::new(node.center.x - new_halfsize, node.center.y + new_halfsize),
                _ => panic!("Invalid child quadrant"),
            };
        }
        // Get the index which the newly created node will be when pushed
        let idx = self.vec.len();

        match self.vec[node_idx].children[child_quadrant] {
            None => {
                // Empty slot, just push the node and add it to the slot.
                self.vec.push(Node {
                    children: [None; 4],
                    mass,
                    center,
                    center_of_mass: position,
                    half_size: new_halfsize,
                });
                self.vec[node_idx].children[child_quadrant] = Some(idx);
            }
            Some(child_idx) => {
                if self.vec[child_idx].is_leaf() {
                    // We'll be replacing the original leaf node with internal
                    // node, we need to get some information from the original
                    // node beforehand.
                    let original_center_of_mass;
                    let original_mass;
                    let original_half_size;
                    let original_center;
                    {
                        let original = &self.vec[child_idx];
                        original_center_of_mass = original.center_of_mass;
                        original_mass = original.mass;
                        original_half_size = original.half_size;
                        original_center = original.center;
                    }

                    // Push new internal node in the place of the original
                    // leaf node and replace the original node's index in its
                    // parrent with the new one
                    self.vec.push(Node {
                        children: [None; 4],
                        mass: original_mass,
                        center: original_center,
                        center_of_mass: original_center_of_mass,
                        half_size: original_half_size,
                    });
                    self.vec[node_idx].children[child_quadrant] = Some(idx);

                    // Find which quadrant does the original node belong to in
                    // the new internal node and record it in its children there.
                    let new_quadrant = self.vec[idx].get_quadrant(original_center_of_mass);
                    {
                        let (new_node, original) = if child_idx < idx {
                            let (first_half, second_half) = self.vec.split_at_mut(idx);
                            // `idx` is at the beginning of `second_half`
                            (&mut first_half[child_idx], &mut second_half[0])
                        } else {
                            let (first_half, second_half) = self.vec.split_at_mut(child_idx);
                            // `child_idx` is at the beginning of `second_half`
                            (&mut first_half[idx], &mut second_half[0])
                        };

                        // Adjust the half_size and center according to
                        // the quadrant.
                        original.half_size /= 2.;
                        original.center = match new_quadrant {
                            0 => Vec2::new(
                                original.center.x - original.half_size,
                                original.center.y - original.half_size,
                            ),
                            1 => Vec2::new(
                                original.center.x + original.half_size,
                                original.center.y - original.half_size,
                            ),
                            2 => Vec2::new(
                                original.center.x + original.half_size,
                                original.center.y + original.half_size,
                            ),
                            3 => Vec2::new(
                                original.center.x - original.half_size,
                                original.center.y + original.half_size,
                            ),
                            _ => panic!("Invalid quadrant index"),
                        };

                        new_node.children[new_quadrant] = Some(child_idx);
                    }

                    // Try to add the node to the newly created internal node
                    self.split_add_recursive(idx, position, mass);
                } else {
                    // Node is internal, try to add to it
                    self.split_add_recursive(child_idx, position, mass);
                }
            }
        }
    }

    /// Adds the node to the quadtree, subdividing or expanding the tree as
    /// needed
    pub fn add_node(&mut self, position: Vec2, mass: f32) {
        if self.in_bounds(position) {
            self.split_add_recursive(self.root, position, mass);
            return;
        }

        let mut center = self.vec[self.root].center;
        let mut new_bounds = self.bounds;
        // Calculate the half_size of the new boundin box
        let half_size = new_bounds[1].x - new_bounds[0].x;
        let prev_root_idx = self.root;
        let mut children: [Option<usize>; 4] = [None; 4];

        if position.x < center.x {
            if position.y < center.y {
                // To the top-left of the center
                // Expand the bounds accordingly
                new_bounds[0].x = new_bounds[0].x - half_size;
                new_bounds[0].y = new_bounds[0].y - half_size;
                // Note down the previous root as child of ther first one
                children[2] = Some(prev_root_idx);
                // Note down the new center of the root node
                center.y = self.bounds[0].y;
            } else {
                // To the bottom-left of the center
                // Other steps are the same as before
                new_bounds[0].x = new_bounds[0].x - half_size;
                new_bounds[1].y = new_bounds[1].y + half_size;
                children[1] = Some(prev_root_idx);
                center.y = self.bounds[1].y;
            }
            center.x = self.bounds[0].x;
        } else {
            if position.y < center.y {
                // To the top-right of the center
                new_bounds[1].x = new_bounds[1].x + half_size;
                new_bounds[0].y = new_bounds[0].y - half_size;
                children[3] = Some(prev_root_idx);
                center.y = self.bounds[0].y;
            } else {
                // To the bottom-right of the center
                new_bounds[1].x = new_bounds[1].x + half_size;
                new_bounds[1].y = new_bounds[1].y + half_size;
                children[0] = Some(prev_root_idx);
                center.y = self.bounds[1].y;
            }
            center.x = self.bounds[1].x;
        }

        // Calculate new center based on the new bounds
        center.x = (new_bounds[0].x + new_bounds[1].x) / 2.0;
        center.y = (new_bounds[0].y + new_bounds[1].y) / 2.0;

        // Create the new root node
        self.bounds = new_bounds;
        let new_root = self.vec.len();
        self.vec.push(Node {
            children,
            center,
            mass: self.vec[prev_root_idx].mass + mass,
            center_of_mass: ((self.vec[prev_root_idx].mass
                * self.vec[prev_root_idx].center_of_mass)
                + (mass * position))
                / (mass + self.vec[prev_root_idx].mass),
            half_size,
        });
        self.root = new_root;
    }

    /// Calculates the 'theta', which is later used for setting the accuracy.
    fn calculate_theta(&self, node_idx: usize, position: Vec2) -> f32 {
        let node = &self.vec[node_idx];
        let distance = node.center_of_mass.distance(position);
        return (node.half_size * 2.) / distance;
    }

    /// Collect the bodies that can be used to calculate forces on body at
    /// `position`. Only internal nodes with theta value smaller than
    /// `theta_threshold` are returned, otherwise they are expanded until a
    /// leaf node is encountered, which will then be returned.
    pub fn collect_bodies(&mut self, position: Vec2, theta_threshold: f32) -> Vec<&Node> {
        let mut bodies: Vec<&Node> = Vec::new();
        let mut to_visit = vec![self.root];

        while let Some(node_idx) = to_visit.pop() {
            let node = &self.vec[node_idx];
            let theta = self.calculate_theta(node_idx, position);
            if theta < theta_threshold || node.is_leaf() {
                // If node is under the threshold add it to the return vector.
                bodies.push(node);
            } else {
                // Otherwise expand it by adding its children to the visit
                // vector
                for &child in node.children.iter().flatten() {
                    to_visit.push(child);
                }
            }
        }

        return bodies;
    }

    pub fn debug_print(&self, node_idx: usize, indentation: usize) {
        let node = &self.vec[node_idx];
        println!(
            "{}m:{}, com:({}, {})",
            "\t".repeat(indentation),
            node.mass,
            node.center_of_mass.x,
            node.center_of_mass.y
        );
        for child in node.children {
            match child {
                Some(child_idx) => {
                    self.debug_print(child_idx, indentation + 1);
                }
                None => {}
            }
        }
    }
}
