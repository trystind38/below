use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};



#[derive(Debug, Clone, Copy, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position {
            x: x,
            y: y,
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
        && self. y == other.y
    }
}

impl Eq for Position {}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.x.cmp(&other.x)
        .then(self.y.cmp(&other.y))
    }
}



#[derive(Debug, Clone, Copy, Hash)]
pub struct NavNode {
    pub position: Position,
    pub g_cost: i32,
    pub f_score: i32,
}

impl NavNode {
    pub fn new(position: Position, g_cost: i32, f_score: i32) -> NavNode {
        NavNode {
            position: position,
            g_cost: g_cost,
            f_score: f_score,
        }
    }

    fn f_score(&self) -> i32 {
        self.g_cost + self.f_score
    }
}

impl PartialEq for NavNode {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
        && self.g_cost == other.g_cost
        && self.f_score == other.g_cost
    }
}

impl Eq for NavNode {}

impl PartialOrd for NavNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))   
    }
}

impl Ord for NavNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f_score().cmp(&other.f_score())
        .then(self.position.cmp(&other.position))
    }
}



// The position and cost of traversal
type Neighbors = Vec<(Position, i32)>;

// The NodeGraph contains every NavNode, represented by their Position, paired with their list of neighbors
struct NodeGraph {
    neighbors: HashMap<Position, Neighbors>
}



fn manhattan_distance_heuristic(from: Position, to: Position) -> i32 {
    (from.x - to.x).abs()
    + (from.y - to.y).abs()
}



type Path = Vec<Position>;

fn astar_path(start: Position, destination: Position, graph: &NodeGraph) -> Option<Path> {
    let mut open = BinaryHeap::new();
    let mut parents: HashMap<Position, Position> = HashMap::new();
    let mut g_costs: HashMap<Position, i32> = HashMap::new();

    // Start by inserting the current start position into the g_costs HashMap "log" with a cost of 0.
    g_costs.insert(start, 0);

    // The open BinaryHeap tracks what nodes are being explored for the path.
    open.push(
        NavNode {
            position: start,
            g_cost: 0,
            f_score: manhattan_distance_heuristic(start, destination)
        }
    );

    // While there are "nodes" in that open BinaryHeap,
    while let Some(
        NavNode {
            position: mut current,
            g_cost,
            ..
        }
    ) = open.pop() {
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
                    open.push(NavNode::new(
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

// // Example pathfinding with a very simple grid
// fn main() {
//     let node1 = Position::new(0,0);
//     let node2 = Position::new(0,1);
//     let node3 = Position::new(1,0);
//     let node4 = Position::new(1,1);

//     let graph = NodeGraph {
//         neighbors: HashMap::from([
//             (node1, vec![(node2, 1), (node3, 1)]),
//             (node2, vec![(node1, 1), (node4, 1)]),
//             (node3, vec![(node1, 1), (node4, 1)]),
//             (node4, vec![(node2, 1), (node3, 1)]),
//         ])
//     };

//     let start = node1;
//     let end = node4;

//     // All that is needed is a start Position, destination Position, and a graph of neighbors: Vec<(Position, i32)>
//     match astar_path(start, end, &graph) {
//         Some(path) => println!("Path found: {:#?}", path),
//         None => println!("No path found"),
//     }
// }
