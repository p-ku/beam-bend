extern crate nalgebra as na;
use crate::build::Section;
mod build;
/* mod load;
mod position; */

fn main() {
    /*     let x: [f64; 6] = [-0.5, 0.0, 0.5, -0.5, 0.0, 0.5];
    let y: [f64; 6] = [0.5, 0.5, 0.5, -0.5, -0.5, -0.5]; */

    /*     let mut j = Section::new(0.3);
    j.add_node(1., 2.); */
    let mut x = Section::new(0.3);
    let a = x.add_node(-9., 9.);
    let b = x.add_node(0.0, 9.);
    let c = x.add_node(9., 9.);
    let d = x.add_node(-9., -9.);
    let e = x.add_node(0.0, -9.);
    let f = x.add_node(9., -9.);
    let _ab = x.add_edge(a, b, None, None);
    let _bc = x.add_edge(b, c, None, None);
    let _be = x.add_edge(b, e, None, None);
    let _de = x.add_edge(d, e, None, None);
    let _ef = x.add_edge(e, f, None, None);
    /*     let a = x.add_node(1., 0.);
    let b = x.add_node(1. / 2., (3f64).sqrt() / 2.);
    let c = x.add_node(-1. / 2., (3f64).sqrt() / 2.);
    let d = x.add_node(-1., 0.);
    let e = x.add_node(-1. / 2., -(3f64).sqrt() / 2.);
    let f = x.add_node(1. / 2., -(3f64).sqrt() / 2.);

    let _ab = x.add_edge(a, b, None, None);
    let _bc = x.add_edge(b, c, None, None);
    let _cd = x.add_edge(c, d, None, None);
    let _de = x.add_edge(d, e, None, None);
    let _ef = x.add_edge(e, f, None, None);
    let _fa = x.add_edge(f, a, None, None); */

    x.build();
}
