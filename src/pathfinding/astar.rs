/*  astar.rs contains the general implementation of the A* search algorithm.
 *
 *  It allows for solving for a path between 2 points, given the graph those 2 points are on.
 *
 *  To adapt to a platformer, that work is done outside of this general implementation.
 *  It doesn't need to know how neighbors and traversal costs are calculated, it just needs to be provided the 2 points and graph.
 */

use bevy::prelude::IVec2;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone, Copy, Hash)]
pub struct Node {
    pub position: IVec2,
    pub g_cost: i32,
    pub f_score: i32,
}

impl Node {
    pub fn new(position: IVec2, g_cost: i32, f_score: i32) -> Node {
        Node {
            position: position,
            g_cost: g_cost,
            f_score: f_score,
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
        && self.g_cost == other.g_cost
        && self.f_score == other.f_score
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
        .then(other.position.x.cmp(&self.position.x))
        .then(other.position.y.cmp(&self.position.y))
    }
}

// The position and cost of traversal
pub type Neighbors = Vec<(IVec2, i32)>;

// The NodeGraph contains every Node, represented by their IVec2, paired with their list of neighbors
pub struct NodeGraph {
    pub neighbors: HashMap<IVec2, Neighbors>
}

impl NodeGraph {
    pub fn new(neighbors: HashMap<IVec2, Neighbors>) -> NodeGraph {
        NodeGraph {
            neighbors: neighbors,
        }
    }
}

fn manhattan_distance_heuristic(from: IVec2, to: IVec2) -> i32 {
    (from.x - to.x).abs() + (from.y - to.y).abs()
}

pub type Path = Vec<IVec2>;

pub fn astar_path(start: IVec2, destination: IVec2, graph: &NodeGraph) -> Option<Path> {
    let mut open = BinaryHeap::new();
    let mut parents: HashMap<IVec2, IVec2> = HashMap::new();
    let mut g_costs: HashMap<IVec2, i32> = HashMap::new();

    // Start by inserting the current start position into the g_costs HashMap "log" with a cost of 0.
    g_costs.insert(start, 0);

    // The open BinaryHeap tracks what nodes are being explored for the path.
    open.push(Node {
        position: start,
        g_cost: 0,
        f_score: manhattan_distance_heuristic(start, destination),
    });

    // While there are "nodes" in that open BinaryHeap,
    while let Some(Node {
        position: mut current,
        g_cost,
        ..
    }) = open.pop()
    {
        // If the current node's g cost this time around is the greatest, we skip searching this node
        if g_cost > g_costs.get(&current).copied().unwrap_or(i32::MAX) {
            continue;
        }

        // If we're at the destination,
        if current == destination {
            let mut path: Path = vec![current];

            // construct the path that led us here and return it
            while let Some(&previous) = parents.get(&current) {
                path.push(previous);
                current = previous;
            }
            path.reverse();
            return Some(path);
        }

        // If not at the destination and the base cases were false:

        // check current position's neighboring positions
        if let Some(neighbors) = graph.neighbors.get(&current) {
            // For every neighbor,
            for &(neighbor_position, neighbor_cost) in neighbors {
                let new_g = g_cost + neighbor_cost;
                let current_g = g_costs.get(&neighbor_position).copied().unwrap_or(i32::MAX);

                // The neighbor is the way to go if the new g cost to it is less
                if new_g < current_g {
                    parents.insert(neighbor_position, current);
                    g_costs.insert(neighbor_position, new_g);
                    open.push(Node::new(
                        neighbor_position,
                        new_g,
                        new_g + manhattan_distance_heuristic(neighbor_position, destination),
                    ));
                }
            }
        }
    }

    None
}
