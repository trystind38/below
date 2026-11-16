use crate::proc_gen::geometry::Point;
use bevy::prelude::*;

pub struct Triangle {
    pub point_a: Point,
    pub point_b: Point,
    pub point_c: Point,
}

pub struct QuarterEdge {
    pub data: Option<Point>,
    pub next: usize,
    pub rot: usize,
}

impl Triangle {
    pub fn new(a: Point, b: Point, c: Point) -> Self {
        Self {
            point_a: a,
            point_b: b,
            point_c: c,
        }
    }

    pub fn square_matrix_determinant(&self, p: Point) -> f64 {
        let (axpx, aypy, a_total) = (
            self.point_a.x - p.x,
            self.point_a.y - p.y,
            (self.point_a.x * self.point_a.x + self.point_a.y * self.point_a.y)
                - (p.x * p.x + p.y * p.y),
        );
        let (bxpx, bypy, b_total) = (
            self.point_b.x - p.x,
            self.point_b.y - p.y,
            (self.point_b.x * self.point_b.x + self.point_b.y * self.point_b.y)
                - (p.x * p.x + p.y * p.y),
        );
        let (cxpx, cypy, c_total) = (
            self.point_c.x - p.x,
            self.point_c.y - p.y,
            (self.point_c.x * self.point_c.x + self.point_c.y * self.point_c.y)
                - (p.x * p.x + p.y * p.y),
        );

        return axpx * (bypy * c_total - b_total * cypy) - aypy * (bxpx * c_total - cxpx * b_total)
            + a_total * (bxpx * cypy - cxpx * bypy);
    }
}

pub fn rot(edges: &Vec<QuarterEdge>, e: usize) -> usize {
    return edges[e].rot;
}

pub fn sym(edges: &Vec<QuarterEdge>, e: usize) -> usize {
    return rot(edges, rot(edges, e));
}

pub fn tor(edges: &Vec<QuarterEdge>, e: usize) -> usize {
    return rot(edges, rot(edges, rot(edges, e)));
}

pub fn prev(edges: &Vec<QuarterEdge>, e: usize) -> usize {
    return rot(edges, edges[rot(edges, e)].next);
}

pub fn lnext(edges: &Vec<QuarterEdge>, e: usize) -> usize {
    return rot(edges, edges[tor(edges, e)].next);
}
pub fn dest(edges: &Vec<QuarterEdge>, e: usize) -> Option<Point> {
    return edges[sym(edges, e)].data.clone();
}

pub fn makeQuadEdge(edges: &mut Vec<QuarterEdge>, start: Point, end: Point) -> usize {
    let start_end = edges.len(); // first address
    let left_right = start_end + 1; // next
    let end_start = left_right + 1; // next
    let right_left = end_start + 1; // next

    //start end
    edges.push(QuarterEdge {
        data: Some(start),
        next: start_end,
        rot: left_right,
    });
    //left right
    edges.push(QuarterEdge {
        data: None,
        next: right_left,
        rot: end_start,
    });
    //end start
    edges.push(QuarterEdge {
        data: Some(end),
        next: end_start,
        rot: right_left,
    });
    //right left
    edges.push(QuarterEdge {
        data: None,
        next: left_right,
        rot: start_end,
    });

    return start_end;
}

pub fn swap_nexts(edges: &mut Vec<QuarterEdge>, thing1: usize, thing2: usize) {
    //const anext = a.next;
    let hold = edges[thing1].next;

    //a.next = b.next;
    edges[thing1].next = edges[thing2].next;

    //b.next = anext;
    edges[thing2].next = hold
}

pub fn splice(edges: &mut Vec<QuarterEdge>, thing1: usize, thing2: usize) {
    //swapNexts(a.next.rot, b.next.rot);
    let thing1_next_rot = rot(edges, edges[thing1].next);
    let thing2_next_rot = rot(edges, edges[thing2].next);
    swap_nexts(edges, thing1_next_rot, thing2_next_rot);

    //swapNexts(a, b);
    swap_nexts(edges, thing1, thing2);
}

pub fn make_triangle(edges: &mut Vec<QuarterEdge>, a: Point, b: Point, c: Point) -> usize {
    //const ab = makeQuadEdge(a, b);
    //const bc = makeQuadEdge(b, c);
    //const ca = makeQuadEdge(c, a);
    let ab = makeQuadEdge(edges, a, b);
    let bc = makeQuadEdge(edges, b, c);
    let ca = makeQuadEdge(edges, c, a);

    //splice(ab.sym, bc);
    //splice(bc.sym, ca);
    //splice(ca.sym, ab);
    let ab_sym = sym(edges, ab);
    let bc_sym = sym(edges, bc);
    let ca_sym = sym(edges, ca);
    splice(edges, ab_sym, bc);
    splice(edges, bc_sym, ca);
    splice(edges, ca_sym, ab);

    return ab;
}

pub fn connect(edges: &mut Vec<QuarterEdge>, a: usize, b: usize) -> usize {
    //const new_edge = makeQuadEdge(a.dest, b.data);
    let adest = match dest(edges, a) {
        Some(point) => point,
        None => return a,
    };
    let bdata = match edges[b].data {
        Some(point) => point,
        None => return b,
    };
    let new_edge = makeQuadEdge(edges, adest, bdata);

    //splice(new_edge, a.lnext);
    let alnext = lnext(edges, a);
    splice(edges, new_edge, alnext);

    //splice(new_edge.sym, b);
    let new_edge_sym = sym(edges, new_edge);
    splice(edges, new_edge_sym, b);

    return new_edge;
}

pub fn sever(edges: &mut Vec<QuarterEdge>, edge: usize) {
    //splice(edge, edge.prev);
    let edge_prev = prev(edges, edge);
    splice(edges, edge, edge_prev);

    //splice(edge.sym, edge.sym.prev);
    let edge_sym = sym(edges, edge);
    let edge_sym_prev = prev(edges, edge_sym);
    splice(edges, edge_sym, edge_sym_prev);
}

pub fn insert_point(edges: &mut Vec<QuarterEdge>, mut polygon_edge: usize, p: Point) -> usize {
    //const first_spoke = makeQuadEdge(polygon_edge.data, point);
    let polygon_edge_data = match edges[polygon_edge].data {
        Some(point) => point,
        None => return polygon_edge,
    };
    let first_spoke = makeQuadEdge(edges, polygon_edge_data, p);

    // splice(first_spoke, polygon_edge);
    splice(edges, first_spoke, polygon_edge);

    // let spoke = first_spoke;
    let mut spoke = first_spoke;

    loop {
        //spoke = connect(polygon_edge, spoke.sym);
        let spoke_sym = sym(edges, spoke);
        spoke = connect(edges, polygon_edge, spoke_sym);

        //spoke.rot.data = null;
        let spoke_rot = rot(edges, spoke);
        edges[spoke_rot].data = None;

        //spoke.tor.data = null;
        let spoke_tor = tor(edges, spoke);
        edges[spoke_tor].data = None;

        //polygon_edge = spoke.prev;
        polygon_edge = prev(edges, spoke);
        let polygon_edge_lnext = lnext(edges, polygon_edge);
        if polygon_edge_lnext == first_spoke {
            break;
        }
    }
    return first_spoke;
}

pub fn flip(edges: &mut Vec<QuarterEdge>, edge: usize) {
    //const a = edge.prev;
    let a = prev(edges, edge);
    let a_dest = dest(edges, a);

    //const b = edge.sym.prev;
    let edge_sym = sym(edges, edge);
    let b = prev(edges, edge_sym);
    let b_dest = dest(edges, b);

    //splice(edge, a);
    let a_lnext = lnext(edges, a);
    splice(edges, edge, a);

    //splice(edge.sym, b);
    let b_lnext = lnext(edges, b);
    splice(edges, edge_sym, b);

    //splice(edge, a.lnext);
    splice(edges, edge, a_lnext);

    //splice(edge.sym, b.lnext);
    splice(edges, edge_sym, b_lnext);

    //edge.data = a.dest;
    edges[edge].data = a_dest;

    //edge.dest = b.dest;
    edges[edge_sym].data = b_dest;
}

pub fn should_flip(edges: &Vec<QuarterEdge>, edge: usize, new_point: Point) -> bool {
    let across_edge = prev(edges, edge);
    let opposite_point = match dest(edges, across_edge) {
        Some(point) => point,
        None => unreachable!(),
    };
    let edge_start = match edges[edge].data {
        Some(point) => point,
        None => unreachable!(),
    };
    let edge_end = match dest(edges, edge) {
        Some(point) => point,
        None => unreachable!(),
    };

    let start_is_corner = edge_start.x.abs() > 1000.0 || edge_start.y.abs() > 1000.0;
    let end_is_corner = edge_end.x.abs() > 1000.0 || edge_end.y.abs() > 1000.0;
    let opposite_is_corner = opposite_point.x.abs() > 1000.0 || opposite_point.y.abs() > 1000.0;

    //1. If the edge in question is a boundary edge, do not flip it. This one is easy so I’m not going to draw it for you.
    if start_is_corner && end_is_corner {
        return false;
    }
    //2. If the edge contains a boundary vertex, flip it, unless doing so would create an inside-out triangle.
    if start_is_corner || end_is_corner {
        let to_opposite_x = opposite_point.x - new_point.x;
        let to_opposite_y = opposite_point.y - new_point.y;

        let start_side = to_opposite_x * (edge_start.y - new_point.y)
            - to_opposite_y * (edge_start.x - new_point.x);

        let end_side =
            to_opposite_x * (edge_end.y - new_point.y) - to_opposite_y * (edge_end.x - new_point.x);

        return start_side * end_side < 0.0;
    }

    //3. If an edge separates our newly inserted point from a boundary vertex, do not flip it.
    if opposite_is_corner {
        return false;
    }

    let triangle = Triangle::new(edge_start, opposite_point, edge_end);
    return triangle.square_matrix_determinant(new_point) > 0.0;
}

pub fn right_of(edges: &Vec<QuarterEdge>, p: Point, current_edge: usize) -> bool {
    let edge_start = match edges[current_edge].data {
        Some(point) => point,
        None => unreachable!(),
    };
    let edge_end = match dest(edges, current_edge) {
        Some(point) => point,
        None => unreachable!(),
    };

    //checking matrix determinant sign for each traingle
    return (p.x * (edge_end.y - edge_start.y) - p.y * (edge_end.x - edge_start.x)
        + (edge_end.x * edge_start.y - edge_end.y * edge_start.x))
        > 0.0;
}

pub fn legalize(edges: &mut Vec<QuarterEdge>, first_spoke: usize, new_point: Point) {
    let mut ring_edge = sym(edges, edges[first_spoke].next);

    loop {
        if should_flip(edges, ring_edge, new_point) {
            flip(edges, ring_edge);
            ring_edge = prev(edges, ring_edge);
        } else if edges[ring_edge].next == first_spoke {
            break;
        } else {
            let next_around_start = edges[ring_edge].next;
            ring_edge = sym(edges, edges[next_around_start].next);
        }
    }
}

pub fn find_bounding_triangle(edges: &Vec<QuarterEdge>, p: Point) -> usize {
    let mut current_edge = 0;
    loop {
        let edge_start = match edges[current_edge].data {
            Some(point) => point,
            None => unreachable!(),
        };
        let edge_end = match dest(edges, current_edge) {
            Some(point) => point,
            None => unreachable!(),
        };

        let wall1 = current_edge;
        let wall2 = edges[current_edge].next;
        let wall3 = sym(edges, lnext(edges, current_edge));

        // If it's on a starting point, then return
        if (p.x == edge_start.x && p.y == edge_start.y) || (p.x == edge_end.x && p.y == edge_end.y)
        {
            return current_edge;
        } else if right_of(edges, p, wall1) {
            current_edge = sym(edges, wall1);
        } else if !right_of(edges, p, wall2) {
            current_edge = wall2;
        } else if !right_of(edges, p, wall3) {
            current_edge = wall3;
        } else {
            return current_edge;
        }
    }
}
