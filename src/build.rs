// Heavily based on https://crates.io/crates/trussx
/* use geo::prelude::{Area, Centroid, MapCoords};
 */
use geo::{polygon, LineString, MultiPolygon, Point, Polygon};
use geo_booleanop::boolean::BooleanOp;
use geojson::{Feature, GeoJson, Geometry as JsonGeometry, Value};
use petgraph::graph::{EdgeIndex, NodeIndex, UnGraph};
use std::convert::From;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct Node {
    /// The position of the node
    x: f32,
    y: f32,
}

pub struct Element {
    thickness: f32,
    elastic: f32,
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({}, {})", self.thickness, self.elastic)
    }
}
pub struct State {
    centroid: Point<f32>,
    second_moment_area: f32,
}

pub struct Section {
    /// A graph structure containing most of the information about the section
    pub graph: UnGraph<Node, Element>,
    poisson: f32,
    /// A bool indicating whether or not results are current
    results: bool,
}

impl Section {
    /// This function instantiates a section
    pub fn new(poisson: f32) -> Section {
        Section {
            graph: petgraph::Graph::new_undirected(),
            poisson,
            results: false,
        }
    }
    pub fn add_node(&mut self, x: f32, y: f32) -> NodeIndex {
        self.graph.add_node(Node { x, y })
    }
    /// This function creates a new element to connect two nodes
    pub fn add_edge(
        &mut self,
        a: NodeIndex,
        b: NodeIndex,
        thickness: Option<f32>,
        elastic: Option<f32>,
    ) -> Option<EdgeIndex> {
        if !intersects(self, a, b) {
            Some(self.graph.add_edge(
                a,
                b,
                Element {
                    thickness: thickness.unwrap_or(0.1),
                    elastic: elastic.unwrap_or(29000.),
                },
            ))
        } else {
            panic!("Don't cross the streams.");
        }
    }

    /// This function moves a node
    pub fn move_node(&mut self, a: NodeIndex, x: f32, y: f32) {
        let node = self.graph.node_weight_mut(a);
        match node {
            None => {
                panic!("This node does not exist.");
            }
            Some(node) => {
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
        let clusters: Vec<Polygon<f32>> = self
            .graph
            .node_indices()
            .map(|node| {
                let niter = self.graph.neighbors(node);

                /*                 let mut npoints: Vec<[f32; 2]> = piter
                    .map(|neigh| {
                        [
                            self.graph.node_weight(neigh).unwrap().x,
                            self.graph.node_weight(neigh).unwrap().y,
                        ]
                    })
                    .collect();
                let mut thick: Vec<f32> = thiter
                    .map(|neigh| {
                        self.graph
                            .edge_weight(self.graph.find_edge(node, neigh).unwrap())
                            .unwrap()
                            .thickness
                    })
                    .collect(); */
                let mut neiter: Vec<(f32, [f32; 2], [f32; 2], f32, NodeIndex)> = niter
                    .map(|neigh| {
                        let x1 = self.graph.node_weight(node).unwrap().x;
                        let y1 = self.graph.node_weight(node).unwrap().y;
                        let x2 = self.graph.node_weight(neigh).unwrap().x;
                        let y2 = self.graph.node_weight(neigh).unwrap().y;
                        let angle = (y2 - y1 / x2 - x1).atan2(0.);
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
                neiter.append(&mut vec![neiter[0]]);
                neiter.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                let mut points = vec![];
                println!("{:?}", neiter);
                for count in 0..(neiter.len() - 1) {
                    let edge = self.graph.find_edge(node, neiter[count].4).unwrap();
                    let ends = self.graph.edge_endpoints(edge).unwrap();
                    let t = neiter[count].3 / 2.;
                    let t2 = neiter[count + 1].3 / 2.;
                    /*                     let x1 = self.graph.node_weight(node).unwrap().x;
                    let y1 = self.graph.node_weight(node).unwrap().y;
                    let x2 = self.graph.node_weight(neigh).unwrap().x;
                    let y2 = self.graph.node_weight(neigh).unwrap().y; */
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
                    /*                     let normal2 = normalize((x1 + x2) / 2., (y1 + y2) / 2., x1, y1);
                     */
                    let d = (t2 * t2 - t * t).abs().sqrt();
                    /*                     println!("{}", d);
                     */
                    println!("1({}, {}) 2({}, {})", x1, y1, x2, y2);
                    /*                    println!(
                        "norm 1({}, {}) 2({}, {})",
                        normal1[0], normal1[1], normal2[0], normal2[1]
                    ); */
                    let neigh1 = (
                        midx + (normal2[1] - normal2[0]) * t,
                        midy + (-normal2[0] - normal2[1]) * t,
                    );
                    let neigh2 = (
                        midx + (-normal2[1] - normal2[0]) * t,
                        midy + (normal2[0] - normal2[1]) * t,
                    );

                    let near1 = (
                        x1 + (normal1[1] - normal1[0]) * t,
                        y1 + (-normal1[0] - normal1[1]) * t,
                    );
                    let near2 = (
                        x1 + (normal3[1] + normal3[0]) * t2,
                        y1 + (-normal3[0] + normal3[1]) * t2,
                    );
                    let far = (
                        x3 + (normal3[1] - normal3[0]) * t2,
                        y3 + (-normal3[0] - normal3[1]) * t2,
                    );

                    let one = near1;
                    let two = neigh2;
                    let thr = near2;
                    let fou = far;

                    /*                     if !intersects2([one, two, thr, fou]) {
                        println!("Lines don't intersect");
                    } */
                    let denom =
                        (one.0 - two.0) * (thr.1 - fou.1) - (one.1 - two.1) * (thr.0 - fou.0);
                    let part1 = one.0 * two.1 - one.1 * two.0;
                    let part2 = thr.0 * fou.1 - thr.1 * fou.0;
                    if self.graph.neighbors(node).count() > 1 {
                        if (far.1 - near2.1) / (far.0 - near2.0)
                            != (near1.1 - neigh2.1) / (near1.0 - neigh2.0)
                        {
                            let jp = (
                                (part1 * (thr.0 - fou.0) - (one.0 - two.0) * part2) / denom,
                                (part1 * (thr.1 - fou.1) - (one.1 - two.1) * part2) / denom,
                            );
                            points.append(&mut vec![neigh1, neigh2, jp]);
                        } else {
                            points.append(&mut vec![neigh1, neigh2]);
                        }
                    } else if self.graph.node_count() < 3 {
                        let lone1 = (
                            x1 + (normal1[1] - normal1[0]) * t,
                            y1 + (-normal1[0] - normal1[1]) * t,
                        );
                        let lone2 = (
                            x1 + (-normal2[1] - normal2[0]) * t,
                            y1 + (normal2[0] - normal2[1]) * t,
                        );
                        let lone3 = (
                            x2 + (-normal1[1] - normal1[0]) * t,
                            y2 + (normal1[0] - normal1[1]) * t,
                        );
                        let lone4 = (
                            x2 + (normal2[1] - normal2[0]) * t,
                            y2 + (-normal2[0] - normal2[1]) * t,
                        );
                        points.append(&mut vec![lone1, lone2, lone3, lone4]);
                    }
                }

                println!("{:?}", points);
                Polygon::new(LineString::from(points), vec![])
            })
            .collect();
        println!("{:?}", clusters.len());

        let mut unionized: MultiPolygon<f32> = MultiPolygon(vec![
            polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),],
            polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),],
        ]);
        /*         let unionized: Polygon<f32> = polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),];
         */
        /*         for poly in clusters {
            match poly {
                Some(poly) => unionized = poly.union(&unionized),
                None => {}
            }
        } */
        let test = clusters[4].clone();
        let geojson_polygon: JsonGeometry = JsonGeometry::new(Value::from(&test));

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
pub fn normalize(x0: f32, y0: f32, x1: f32, y1: f32) -> [f32; 2] {
    [
        (x1 - x0) / ((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0)).sqrt(),
        (y1 - y0) / ((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0)).sqrt(),
    ]
}

pub fn intersects(section: &Section, node1: NodeIndex, node2: NodeIndex) -> bool {
    let (mut x3, mut x4, mut y3, mut y4): (f32, f32, f32, f32);
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
                    return true;
                }
            }
            None => println!("No edges."),
        }
    }
    false
}
pub fn intersects2([(x1, y1), (x2, y2), (x3, y3), (x4, y4)]: [(f32, f32); 4]) -> bool {
    let test1 = (x1 - x3) * (y3 - y4)
        - (y1 - y3) * (x3 - x4) / (x1 - x2) * (y3 - y4)
        - (y1 - y2) * (x3 - x4);
    let test2 = (x2 - x1) * (y1 - y3)
        - (y2 - y1) * (x1 - x3) / (x1 - x2) * (y3 - y4)
        - (y1 - y2) * (x3 - x4);
    if test1 >= 0.
        && test1 <= 1.
        && test2 >= 0.
        && test2 <= 1.
        && (x1, y1) != (x3, y3)
        && (x1, y1) != (x4, y4)
        && (x2, y2) != (x3, y3)
        && (x2, y2) != (x4, y4)
    {
        return true;
    }
    false
}
