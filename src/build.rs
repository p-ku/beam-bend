// Heavily based on https://crates.io/crates/trussx
use geo::prelude::{Area, Centroid, MapCoords, Simplify, Translate};
use geo::{polygon, LineString, MultiPolygon, Point, Polygon};
use geo_booleanop::boolean::BooleanOp;
use geojson::{Feature, GeoJson, Geometry as JsonGeometry, Value};
use petgraph::graph::{EdgeIndex, NodeIndex, UnGraph};
use std::convert::From;
use std::f64;
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct Node {
    /// The position of the node
    x: f64,
    y: f64,
}

pub struct Element {
    thickness: f64,
    elastic: f64,
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({}, {})", self.thickness, self.elastic)
    }
}
pub struct State {
    second_moment_area: f64,
    rotation: f64,
}
pub struct Section {
    /// A graph structure containing most of the information about the section
    pub graph: UnGraph<Node, Element>,
    poisson: f64,
    /// A bool indicating whether or not results are current
    results: bool,
}

impl Section {
    /// This function instantiates a section
    pub fn new(poisson: f64) -> Section {
        Section {
            graph: petgraph::Graph::new_undirected(),
            poisson,
            results: false,
        }
    }
    pub fn add_node(&mut self, x: f64, y: f64) -> NodeIndex {
        /*         if point2line()
         */
        if near(self, x, y) {
            panic!("No can do, too close");
        }
        self.graph.add_node(Node { x, y })
    }
    /// This function creates a new element to connect two nodes
    pub fn add_edge(
        &mut self,
        a: NodeIndex,
        b: NodeIndex,
        thickness: Option<f64>,
        elastic: Option<f64>,
    ) -> Option<EdgeIndex> {
        let check = intersects(self, a, b);
        match check {
            Some(points) => {
                let new_node = self.add_node(points.2 .0, points.2 .1);
                self.add_edge(new_node, points.1 .0, Some(points.0), None);
                self.add_edge(new_node, points.1 .1, Some(points.0), None)
            }
            None => {
                return Some(self.graph.add_edge(
                    a,
                    b,
                    Element {
                        thickness: thickness.unwrap_or(2.),
                        elastic: elastic.unwrap_or(29000.),
                    },
                ))
            }
        }
        /*         if !intersects(self, a, b) {
            Some(self.graph.add_edge(
                a,
                b,
                Element {
                    thickness: thickness.unwrap_or(0.1),
                    elastic: elastic.unwrap_or(29000.),
                },
            ))
        } else {
            let one = (
                self.graph.node_weight(a).unwrap().x,
                self.graph.node_weight(a).unwrap().y,
            );

            let intersect = calc_intersect(
                one: (f32, f32),
                two: (f32, f32),
                thr: (f32, f32),
                fou: (f32, f32),
            );
            panic!("Don't cross the streams.");
        } */
    }

    /// This function moves a node
    pub fn move_node(&mut self, a: NodeIndex, x: f64, y: f64) {
        let node = self.graph.node_weight_mut(a);

        match node {
            None => {
                panic!("This node does not exist.");
            }
            Some(node) => {
                /*                 if near(Self, x, y) {
                    panic!("No can do, too close");
                } */
                node.x = x;
                node.y = y;
            }
        }
    }

    /// This function deletes a node
    pub fn delete_node(&mut self, a: NodeIndex) {
        self.graph.remove_node(a);
    }

    /// This function deletes a member
    pub fn delete_member(&mut self, ab: EdgeIndex) {
        self.graph.remove_edge(ab);
    }

    pub fn build(&self) {
        /*         let point_holder = vec![];
         */
        let mut clusters: Vec<Vec<(f64, f64)>> = self
            .graph
            .node_indices()
            .map(|node| {
                let niter = self.graph.neighbors(node);
                let mut neiter: Vec<(f64, [f64; 2], [f64; 2], f64, NodeIndex)> = niter
                    .map(|neigh| {
                        let x1 = self.graph.node_weight(node).unwrap().x;
                        let y1 = self.graph.node_weight(node).unwrap().y;
                        let x2 = self.graph.node_weight(neigh).unwrap().x;
                        let y2 = self.graph.node_weight(neigh).unwrap().y;
                        let angle = (y2 - y1).atan2(x2 - x1);
                        (
                            angle,
                            [x1, y1],
                            [x2, y2],
                            self.graph
                                .edge_weight(self.graph.find_edge(node, neigh).unwrap())
                                .unwrap()
                                .thickness,
                            neigh,
                        )
                    })
                    .collect();
                neiter.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                /*                 neiter.drain(|b:f32| &b.0);
                 */
                neiter.dedup_by(|a, b| {
                    // if collinear
                    if a.0.rem_euclid(f64::consts::PI)
                        == b.0.rem_euclid(f64::consts::PI) - f64::consts::PI / 2.
                        || a.0.rem_euclid(f64::consts::PI)
                            == b.0.rem_euclid(f64::consts::PI) + f64::consts::PI / 2.
                    {
                        // make sure thicknesses are equal
                        assert!(a.3.eq(&b.3));
                        // continue with dedup
                        a.0.eq(&b.0)
                    } else {
                        a.0.eq(&b.0)
                    }
                });

                neiter.append(&mut vec![neiter[0]]);

                let mut points = vec![];
                /*                 println!("{:?}", neiter);
                 */
                for count in 0..(neiter.len() - 1) {
                    let edge = self.graph.find_edge(node, neiter[count].4).unwrap();
                    let t = neiter[count].3 / 2.;
                    let t2 = neiter[count + 1].3 / 2.;

                    let x1 = neiter[count].1[0];
                    let y1 = neiter[count].1[1];
                    let x2 = neiter[count].2[0];
                    let y2 = neiter[count].2[1];
                    let x3 = neiter[count + 1].2[0];
                    let y3 = neiter[count + 1].2[1];

                    let midx = (x1 + x2) / 2.;
                    let midy = (y1 + y2) / 2.;
                    let normal1 = normalize(x1, y1, x2, y2);

                    let normal2 = [-normal1[0], -normal1[1]];
                    let normal3 = normalize(x3, y3, x1, y1);

                    let nei2nod_l = (
                        midx + (-normal2[1] - normal2[0]) * t,
                        midy + (normal2[0] - normal2[1]) * t,
                    );

                    let nei2nod_r = (
                        midx + (normal2[1] - normal2[0]) * t,
                        midy + (-normal2[0] - normal2[1]) * t,
                    );

                    let nod2nei_l = (
                        x1 + (-normal1[1] - normal1[0]) * t,
                        y1 + (normal1[0] - normal1[1]) * t,
                    );
                    let nod2nei_r = (
                        x1 + (normal1[1] - normal1[0]) * t,
                        y1 + (-normal1[0] - normal1[1]) * t,
                    );
                    let nod2nex_r = (
                        x1 + (-normal3[1] + normal3[0]) * t2,
                        y1 + (normal3[0] + normal3[1]) * t2,
                    );
                    let nex2nod_l = (
                        x3 + (-normal3[1] - normal3[0]) * t2,
                        y3 + (normal3[0] - normal3[1]) * t2,
                    );

                    let lone_l = (
                        x2 + (-normal1[1] - normal1[0]) * t,
                        y2 + (normal1[0] - normal1[1]) * t,
                    );
                    let lone_r = (
                        x2 + (normal2[1] - normal2[0]) * t,
                        y2 + (-normal2[0] - normal2[1]) * t,
                    );
                    let one = nod2nei_l;
                    let two = nei2nod_r;
                    let thr = nod2nex_r;
                    let fou = nex2nod_l;

                    /*                     if !intersects2([one, two, thr, fou]) {
                        println!("Lines don't intersect");
                    } */
                    let denom =
                        (one.0 - two.0) * (thr.1 - fou.1) - (one.1 - two.1) * (thr.0 - fou.0);
                    let part1 = one.0 * two.1 - one.1 * two.0;
                    let part2 = thr.0 * fou.1 - thr.1 * fou.0;
                    if self.graph.neighbors(node).count() > 1 {
                        if (nex2nod_l.1 - nod2nex_r.1) / (nex2nod_l.0 - nod2nex_r.0)
                            != (nod2nei_l.1 - nei2nod_r.1) / (nod2nei_l.0 - nei2nod_r.0)
                        {
                            let jp = (
                                (part1 * (thr.0 - fou.0) - (one.0 - two.0) * part2) / denom,
                                (part1 * (thr.1 - fou.1) - (one.1 - two.1) * part2) / denom,
                            );
                            points.append(&mut vec![nei2nod_l, nei2nod_r, jp]);
                        } else {
                            points.append(&mut vec![nei2nod_l, nei2nod_r]);
                        }
                    } else if self.graph.node_count() < 3 {
                        points.append(&mut vec![nod2nei_l, nod2nei_r, lone_l, lone_r]);
                    } else {
                        points.append(&mut vec![nei2nod_l, nei2nod_r, nod2nei_l, nod2nei_r]);
                    }
                }
                println!("{:?}", points);
                points
                /*                     Polygon::new(LineString::from(points) , vec![])
                 */
            })
            .collect::<Vec<_>>();
        clusters.sort_unstable_by(|a, b| a.len().partial_cmp(&b.len()).unwrap().reverse());
        /*          clusters.sort_unstable_by(|a, b| a.exterior().into_points().len().partial_cmp(&b.exterior().into_points().len()).unwrap());
         */
        println!("hey{:?}", clusters);

        let mut unionized: MultiPolygon<f64> = MultiPolygon(vec![]);

        // merge clusters into single polygon
        let mut cccount = 0;
        while cccount < 1 {
            /*             let shape = Polygon::new(LineString::from(clusters[cccount]), vec![]);
             */
            let shape = Polygon::new(LineString::from(clusters[cccount].clone()), vec![]);
            unionized = shape.union(&unionized);
            cccount += 1;
        }
        cccount += 1;
        while cccount < 6 {
            /*             let shape = Polygon::new(LineString::from(clusters[cccount]), vec![]);
             */
            let shape = Polygon::new(LineString::from(clusters[cccount].clone()), vec![]);
            unionized = shape.union(&unionized);
            cccount += 1;
        }
        /*         for poly in clusters {
            unionized = poly.union(&unionized).simplify(&0.01);
        } */

        /*         unionized = unionized.simplify(&0.01);
         */
        let section_poly = unionized.0[0].clone();
        // translate the cross-section to place the centroid at the origin
        let translated = section_poly.translate(
            -section_poly.centroid().unwrap().x(),
            -section_poly.centroid().unwrap().y(),
        );

        let geo_points = translated.into_inner().0.into_points();
        let perimeter_points: Vec<(f64, f64)> =
            geo_points.iter().map(|point| point.x_y()).collect();

        /*            let area = translated.unsigned_area();
         */
        println!("{:#?}", section_poly);

        /*         let test = clusters[4].clone();
         */
        let geojson_polygon: JsonGeometry = JsonGeometry::new(Value::from(&unionized));

        let geojson = GeoJson::Feature(Feature {
            bbox: None,
            geometry: { Some(geojson_polygon) },
            id: None,
            properties: None,
            foreign_members: None,
        });
        let geojson_string = geojson.to_string();
        /*                     println!("{}", geojson_string); */
        let path = Path::new("hello.geojson");
        let display = path.display();
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };
        match file.write_all(geojson_string.as_bytes()) {
            Err(why) => panic!("couldn't write to {}: {}", display, why),
            Ok(_) => println!("successfully wrote to {}", display),
        }
    }
}
pub fn normalize(x0: f64, y0: f64, x1: f64, y1: f64) -> [f64; 2] {
    [
        (x1 - x0) / ((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0)).sqrt(),
        (y1 - y0) / ((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0)).sqrt(),
    ]
}

pub fn intersects(
    section: &Section,
    node1: NodeIndex,
    node2: NodeIndex,
) -> Option<(f64, (NodeIndex, NodeIndex), (f64, f64))> {
    let (mut x3, mut x4, mut y3, mut y4): (f64, f64, f64, f64);
    let x1 = section.graph.node_weight(node1).unwrap().x;
    let y1 = section.graph.node_weight(node1).unwrap().y;
    let x2 = section.graph.node_weight(node2).unwrap().x;
    let y2 = section.graph.node_weight(node2).unwrap().y;
    for edge in section.graph.edge_indices() {
        match section.graph.edge_endpoints(edge) {
            Some(nodes) => {
                x3 = section.graph.node_weight(nodes.0).unwrap().x;
                y3 = section.graph.node_weight(nodes.0).unwrap().y;
                x4 = section.graph.node_weight(nodes.1).unwrap().x;
                y4 = section.graph.node_weight(nodes.1).unwrap().y;
                let test1 = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4))
                    / ((x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4));
                let test2 = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3))
                    / ((x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4));
                if test1 >= 0.
                    && test1 <= 1.
                    && test2 >= 0.
                    && test2 <= 1.
                    && (x1, y1) != (x3, y3)
                    && (x1, y1) != (x4, y4)
                    && (x2, y2) != (x3, y3)
                    && (x2, y2) != (x4, y4)
                {
                    let denom = (x1 - x2) * (y3 - y4) - (x1 - y2) * (x3 - x4);
                    let part1 = x1 * y2 - x1 * x2;
                    let part2 = x3 * y4 - y3 * x4;
                    return Some((
                        section.graph.edge_weight(edge).unwrap().thickness,
                        nodes,
                        (
                            (part1 * (x3 - x4) - (x1 - x2) * part2) / denom,
                            (part1 * (y3 - y4) - (x1 - y2) * part2) / denom,
                        ),
                    ));
                }
            }
            None => {
                println!("No edges.");
            }
        }
    }
    None
}
pub fn near(section: &Section, x0: f64, y0: f64) -> bool {
    let (mut end1, mut end2): ((f64, f64), (f64, f64));
    for edge in section.graph.edge_indices() {
        let nodes = section.graph.edge_endpoints(edge).unwrap();
        end1 = (
            section.graph.node_weight(nodes.0).unwrap().x,
            section.graph.node_weight(nodes.0).unwrap().y,
        );
        end2 = (
            section.graph.node_weight(nodes.1).unwrap().x,
            section.graph.node_weight(nodes.1).unwrap().y,
        );

        if point2line(end1, end2, (x0, y0))
            < 2. * section.graph.edge_weight(edge).unwrap().thickness
        {
            return true;
        }
    }
    false
}

/* pub fn intersects2(
    [(x1, y1), (x2, y2), (x3, y3), (x4, y4)]: [(f64, f64); 4],
) -> Option<(f64, f64)> {
    let test1 = (x1 - x3) * (y3 - y4)
        - (y1 - y3) * (x3 - x4) / (x1 - x2) * (y3 - y4)
        - (y1 - y2) * (x3 - x4);
    let test2 = (x2 - x1) * (y1 - y3)
        - (y2 - y1) * (x1 - x3) / (x1 - x2) * (y3 - y4)
        - (y1 - y2) * (x3 - x4);
    if test1 >= 0. && test1 <= 1. && test2 >= 0. && test2 <= 1.
    /*         && (x1, y1) != (x3, y3)
    && (x1, y1) != (x4, y4)
    && (x2, y2) != (x3, y3)
    && (x2, y2) != (x4, y4) */
    {
        let denom = (x1 - x2) * (y3 - y4) - (x1 - y2) * (x3 - x4);
        let part1 = x1 * y2 - x1 * x2;
        let part2 = x3 * y4 - y3 * x4;
        Some((
            (part1 * (x3 - x4) - (x1 - x2) * part2) / denom,
            (part1 * (y3 - y4) - (x1 - y2) * part2) / denom,
        ))
    /*         return true; */
    } else {
        None
    }

    /*     false
     */
} */
pub fn calc_intersect(
    one: (f32, f32),
    two: (f32, f32),
    thr: (f32, f32),
    fou: (f32, f32),
) -> (f32, f32) {
    let denom = (one.0 - two.0) * (thr.1 - fou.1) - (one.1 - two.1) * (thr.0 - fou.0);
    let part1 = one.0 * two.1 - one.1 * two.0;
    let part2 = thr.0 * fou.1 - thr.1 * fou.0;

    (
        (part1 * (thr.0 - fou.0) - (one.0 - two.0) * part2) / denom,
        (part1 * (thr.1 - fou.1) - (one.1 - two.1) * part2) / denom,
    )
}
fn point2line(end1: (f64, f64), end2: (f64, f64), point: (f64, f64)) -> f64 {
    ((end2.0 - end1.0) * (end1.1 - point.1) - (end1.0 - point.0) * (end2.1 - end1.1)).abs()
        / ((end2.0 - end1.0) * (end2.0 - end1.0) + (end2.1 - end1.1) * (end2.1 - end1.1)).sqrt()
}
