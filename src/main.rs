extern crate nalgebra as na;
use crate::build::Section;
mod build;
/* mod load;
mod position; */

fn main() {
    /*     let x: [f32; 6] = [-0.5, 0.0, 0.5, -0.5, 0.0, 0.5];
    let y: [f32; 6] = [0.5, 0.5, 0.5, -0.5, -0.5, -0.5]; */

    /*     let mut j = Section::new(0.3);
    j.add_node(1., 2.); */
    let mut x = Section::new(0.3);
    let a = x.add_node(-0.5, 0.5);
    let b = x.add_node(0.0, 0.5);
    let c = x.add_node(0.5, 0.5);
    let d = x.add_node(-0.5, 0.4);
    let e = x.add_node(0.0, -0.5);
    let f = x.add_node(0.5, -0.5);
    let _ab = x.add_edge(a, b, Some(0.09), None);
    let _bc = x.add_edge(b, c, None, None);
    let _be = x.add_edge(b, e, None, None);
    let _de = x.add_edge(d, e, None, None);
    let _ef = x.add_edge(e, f, None, None);
    x.build();
}
