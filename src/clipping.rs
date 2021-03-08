// I copy-pasted this code from: https://docs.rs/crate/clipping/0.1.1
use std::fmt;
use std::ptr;

use rand;

#[derive(Debug)]
/// Node in a circular doubly linked list
struct Vertex {
    pt: [f64; 2],
    next: *mut Vertex,
    prev: *mut Vertex,
    neighbour: *mut Vertex,
    entry: bool,
    alpha: f64,
    inter: bool,
    checked: bool,
}

impl Vertex {
    pub fn new(pt: [f64; 2], alpha: f64, inter: bool, entry: bool, checked: bool) -> *mut Self {
        let vertex = Box::new(Self {
            pt: pt,
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
            neighbour: ptr::null_mut(),
            entry: entry,
            alpha: alpha,
            inter: inter,
            checked: checked,
        });
        let ptr = Box::into_raw(vertex);

        unsafe {
            ptr.as_mut().unwrap().next = ptr;
            ptr.as_mut().unwrap().prev = ptr;
        }

        ptr
    }

    pub fn clone(vertex: *mut Vertex) -> *mut Self {
        unsafe {
            Self::new(
                vertex.as_ref().unwrap().pt,
                vertex.as_ref().unwrap().alpha,
                vertex.as_ref().unwrap().inter,
                vertex.as_ref().unwrap().entry,
                vertex.as_ref().unwrap().checked,
            )
        }
    }

    /// Test if the vertex is inside the polygon
    pub fn is_inside(&self, poly: &CPolygon) -> bool {
        let mut w = 0;

        let infinity = [
            -100000. - (rand::random::<f64>() % 100.),
            -100000. - (rand::random::<f64>() % 100.),
        ];

        for q in poly.iter() {
            unsafe {
                let pt_q = q.as_ref().unwrap().pt;
                let pt_q_next = poly.next(q.as_ref().unwrap().next).as_ref().unwrap().pt;

                match intersect(self.pt, infinity, pt_q, pt_q_next) {
                    Some(_) => {
                        if !q.as_ref().unwrap().inter {
                            w += 1;
                        }
                    }
                    None => {}
                }
            }
        }

        w % 2 != 0
    }

    pub fn set_checked(&mut self) {
        self.checked = true;
        if !self.neighbour.is_null() {
            unsafe {
                if !self.neighbour.as_ref().unwrap().checked {
                    self.neighbour.as_mut().unwrap().set_checked();
                }
            }
        }
    }
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let pt = self.pt;
            let npt = self.next.as_ref().unwrap().pt;
            let ppt = self.prev.as_ref().unwrap().pt;

            write!(f, "{:?} <-> {:?} <-> {:?}", ppt, pt, npt)
        }
    }
}

pub struct CPolygon {
    fst: *mut Vertex,
}

impl CPolygon {
    /// Create a new empty polygon
    pub fn new() -> Self {
        Self {
            fst: ptr::null_mut(),
        }
    }

    /// Create a polygon from a list of points
    pub fn from_vec(vec: &Vec<[f64; 2]>) -> Self {
        let mut poly = Self::new();

        for v in vec {
            let vertex = Vertex::new(*v, 0., false, true, false);
            poly.add(vertex);
        }

        poly
    }

    fn iter(&self) -> CPolyIter {
        CPolyIter::new(self)
    }

    /// Add a vertex at the end of the polygon
    fn add(&mut self, vertex: *mut Vertex) {
        if self.fst.is_null() {
            self.fst = vertex;
        } else {
            unsafe {
                let next = self.fst;
                let prev = next.as_ref().unwrap().prev;

                next.as_mut().unwrap().prev = vertex;
                vertex.as_mut().unwrap().next = next;
                vertex.as_mut().unwrap().prev = prev;
                prev.as_mut().unwrap().next = vertex;
            }
        }
    }

    /// Insert and sort a vertex between a specified pair of vertices
    fn insert(&mut self, vertex: *mut Vertex, start: *mut Vertex, end: *mut Vertex) {
        unsafe {
            let mut curr = start;

            while curr != end && curr.as_ref().unwrap().alpha < vertex.as_ref().unwrap().alpha {
                curr = curr.as_ref().unwrap().next;
            }

            vertex.as_mut().unwrap().next = curr;
            let prev = curr.as_ref().unwrap().prev;
            vertex.as_mut().unwrap().prev = prev;
            prev.as_mut().unwrap().next = vertex;
            curr.as_mut().unwrap().prev = vertex;
        }
    }

    /// Return the next non intersecting vertex after the one specified
    fn next(&self, v: *mut Vertex) -> *mut Vertex {
        unsafe {
            let mut c = v;
            while c.as_ref().unwrap().inter {
                c = c.as_ref().unwrap().next;
            }
            c
        }
    }

    /// Return the first unchecked intersection point in the polygon
    fn first_intersect(&self) -> *mut Vertex {
        let mut ve = ptr::null_mut();

        for v in self.iter() {
            ve = v;
            unsafe {
                if v.as_ref().unwrap().inter && !v.as_ref().unwrap().checked {
                    break;
                }
            }
        }

        ve
    }

    /// Return the polygon points as a list of points, clear consecutive equals points
    pub fn points(&self) -> Vec<[f64; 2]> {
        let mut poly = vec![];

        for vertex in self.iter() {
            unsafe {
                let pt = vertex.as_ref().unwrap().pt;
                if vertex == self.fst {
                    poly.push(pt);
                } else {
                    if vertex == self.fst.as_ref().unwrap().prev {
                        if self.fst.as_ref().unwrap().pt != pt {
                            poly.push(pt);
                        }
                    } else {
                        let prev = vertex.as_ref().unwrap().prev.as_ref().unwrap().pt;

                        if prev != pt {
                            poly.push(pt);
                        }
                    }
                }
            }
        }

        poly
    }

    /// Check if an unchecked intersection remain in the polygon
    fn unprocessed(&self) -> bool {
        for v in self.iter() {
            unsafe {
                if v.as_ref().unwrap().inter && !v.as_ref().unwrap().checked {
                    return true;
                }
            }
        }

        false
    }

    fn phase_one(&mut self, poly: &mut CPolygon) {
        for s in self.iter() {
            unsafe {
                if !s.as_ref().unwrap().inter {
                    for c in poly.iter() {
                        if !c.as_ref().unwrap().inter {
                            let s_pt = s.as_ref().unwrap().pt;
                            let s_next_pt =
                                self.next(s.as_ref().unwrap().next).as_ref().unwrap().pt;
                            let c_pt = c.as_ref().unwrap().pt;
                            let c_next_pt =
                                poly.next(c.as_ref().unwrap().next).as_ref().unwrap().pt;

                            match intersect(s_pt, s_next_pt, c_pt, c_next_pt) {
                                Some((i, alpha_s, alpha_c)) => {
                                    let mut is = Vertex::new(i, alpha_s, true, false, false);
                                    let mut ic = Vertex::new(i, alpha_c, true, false, false);
                                    is.as_mut().unwrap().neighbour = ic;
                                    ic.as_mut().unwrap().neighbour = is;

                                    let s_next = self.next(s.as_ref().unwrap().next);
                                    self.insert(is, s, s_next);

                                    let c_next = poly.next(c.as_ref().unwrap().next);
                                    poly.insert(ic, c, c_next);
                                }
                                None => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn phase_two(&mut self, poly: &mut CPolygon, mut s_entry: bool, mut c_entry: bool) {
        unsafe {
            s_entry ^= self.fst.as_ref().unwrap().is_inside(poly);
            for s in self.iter() {
                if s.as_ref().unwrap().inter {
                    s.as_mut().unwrap().entry = s_entry;
                    s_entry = !s_entry;
                }
            }

            c_entry ^= poly.fst.as_ref().unwrap().is_inside(self);
            for c in poly.iter() {
                if c.as_ref().unwrap().inter {
                    c.as_mut().unwrap().entry = c_entry;
                    c_entry = !c_entry;
                }
            }
        }
    }

    fn phase_three(&mut self) -> Vec<Vec<[f64; 2]>> {
        let mut list = vec![];

        while self.unprocessed() {
            unsafe {
                let mut curr = self.first_intersect();
                let mut clipped = CPolygon::new();
                clipped.add(Vertex::clone(curr));

                loop {
                    curr.as_mut().unwrap().set_checked();
                    if curr.as_ref().unwrap().entry {
                        loop {
                            curr = curr.as_ref().unwrap().next;
                            clipped.add(Vertex::clone(curr));
                            if curr.as_ref().unwrap().inter {
                                break;
                            }
                        }
                    } else {
                        loop {
                            curr = curr.as_ref().unwrap().prev;
                            clipped.add(Vertex::clone(curr));
                            if curr.as_ref().unwrap().inter {
                                break;
                            }
                        }
                    }

                    curr = curr.as_ref().unwrap().neighbour;
                    if curr.as_ref().unwrap().checked {
                        break;
                    }
                }

                list.push(clipped.points());
            }
        }

        if list.is_empty() {
            let mut clipped = vec![];
            for s in self.iter() {
                unsafe {
                    clipped.push(s.as_ref().unwrap().pt);
                }
            }
            list.push(clipped);
        }

        list
    }

    pub fn clip(
        &mut self,
        poly: &mut CPolygon,
        s_entry: bool,
        c_entry: bool,
    ) -> Vec<Vec<[f64; 2]>> {
        self.phase_one(poly);
        self.phase_two(poly, s_entry, c_entry);
        self.phase_three()
    }

    pub fn union(&mut self, poly: &mut CPolygon) -> Vec<Vec<[f64; 2]>> {
        self.clip(poly, false, false)
    }

    pub fn intersection(&mut self, poly: &mut CPolygon) -> Vec<Vec<[f64; 2]>> {
        self.clip(poly, true, true)
    }

    pub fn difference(&mut self, poly: &mut CPolygon) -> Vec<Vec<[f64; 2]>> {
        self.clip(poly, false, true)
    }
}

impl Drop for CPolygon {
    fn drop(&mut self) {
        let mut curr = self.fst;

        if !curr.is_null() {
            loop {
                unsafe {
                    let next = curr.as_ref().unwrap().next;
                    Box::from_raw(curr);
                    curr = next;
                    if curr == self.fst {
                        break;
                    }
                }
            }
        }
    }
}

struct CPolyIter {
    first: *mut Vertex,
    curr: *mut Vertex,
    fst: bool,
}

impl CPolyIter {
    pub fn new(poly: &CPolygon) -> Self {
        Self {
            first: poly.fst,
            curr: poly.fst,
            fst: true,
        }
    }
}

impl Iterator for CPolyIter {
    type Item = *mut Vertex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fst {
            if self.curr.is_null() {
                None
            } else {
                let i = self.curr;
                unsafe {
                    self.curr = self.curr.as_ref().unwrap().next;
                }
                self.fst = false;
                Some(i)
            }
        } else {
            if self.curr == self.first {
                None
            } else {
                let i = self.curr;
                unsafe {
                    self.curr = self.curr.as_ref().unwrap().next;
                }
                Some(i)
            }
        }
    }
}

/// Test the intersection of two line and get the intersection point and alphas
fn intersect(
    s1: [f64; 2],
    s2: [f64; 2],
    c1: [f64; 2],
    c2: [f64; 2],
) -> Option<([f64; 2], f64, f64)> {
    let den = (c2[1] - c1[1]) * (s2[0] - s1[0]) - (c2[0] - c1[0]) * (s2[1] - s1[1]);

    if den == 0. {
        return None;
    }

    let us = ((c2[0] - c1[0]) * (s1[1] - c1[1]) - (c2[1] - c1[1]) * (s1[0] - c1[0])) / den;
    let uc = ((s2[0] - s1[0]) * (s1[1] - c1[1]) - (s2[1] - s1[1]) * (s1[0] - c1[0])) / den;

    if (us == 0. || us == 1.) && (0. <= uc && uc <= 1.)
        || (uc == 0. || uc == 1.) && (0. <= us && us <= 1.)
    {
        // degenerate case
        return None;
    } else if (0. < us && us < 1.) && (0. < uc && uc < 1.) {
        let x = s1[0] + us * (s2[0] - s1[0]);
        let y = s1[1] + us * (s2[1] - s1[1]);

        let pt = [x, y];
        return Some((pt, us, uc));
    }

    None
}
