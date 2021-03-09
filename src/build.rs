// Heavily based on https://crates.io/crates/trussx
/* use geo::prelude::{Area, Centroid, MapCoords};
 */
use geo::{polygon, LineString, MultiPolygon, Point, Polygon};
use geo_booleanop::boolean::BooleanOp;
use geojson::{Feature, GeoJson, Geometry as JsonGeometry, Value};
use petgraph::graph::{EdgeIndex, NodeIndex, UnGraph};
use std::convert::From;
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

    /*     pub fn build(&self) {
        let rectangles = self
            .graph
            .edge_indices()
            .map(|edge| match self.graph.edge_endpoints(edge) {
                Some(nodes) => {
                    let x1 = self.graph.node_weight(nodes.0).unwrap().x;
                    let y1 = self.graph.node_weight(nodes.0).unwrap().y;
                    let x2 = self.graph.node_weight(nodes.1).unwrap().x;
                    let y2 = self.graph.node_weight(nodes.1).unwrap().y;
                    let normal1 = normalize(x1, y1, x2, y2);
                    let normal2 = normalize(x2, y2, x1, y1);
                    let thickness = self.graph.edge_weight(edge).unwrap().thickness;
                    let rectangle = polygon![
                        (
                            x: x1 + normal1[1] * thickness,
                            y: y1 - normal1[0] * thickness
                        ),
                        (
                            x: x1 - normal1[1] * thickness,
                            y: y1 + normal1[0] * thickness
                        ),
                        (
                            x: x2 + normal2[1] * thickness,
                            y: y2 - normal2[0] * thickness
                        ),
                        (
                            x: x2 - normal2[1] * thickness,
                            y: y2 + normal2[0] * thickness
                        ),
                    ];
                    /*                     let rectangle = MultiPolygon(vec![rectangle]);
                     */
                    /*                     unionized = rectangle.union(&unionized);
                     */
                    /*                     rectangles.push(rectangle);
                     */
                    rectangle
                }
                None => {
                    println!("No edges");
                    let huh: Polygon<f32> = polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),];
                    huh
                }
            })
            .collect::<MultiPolygon<f32>>();

        println!("{:?}", rectangles);
        let mut unionized: MultiPolygon<f32> = MultiPolygon(vec![
            polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),],
            polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),],
        ]);
        for poly in rectangles {
            /*             self.graph.neighbors(node); */
            unionized = poly.union(&unionized);
        }
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
    } */
    pub fn build(&self) {
        let mut clusters: Vec<Polygon<f32>> = self
            .graph
            .node_indices()
            .map(|node| {
                let niter = self.graph.neighbors(node);
                let mut thick: Vec<f32> = niter
                    .map(|neigh| {
                        self.graph
                            .edge_weight(self.graph.find_edge(node, neigh).unwrap())
                            .unwrap()
                            .thickness
                    })
                    .collect();
                thick.append(&mut vec![thick[0]]);
                let niter2 = self.graph.neighbors(node);
                let mut points = vec![];
                let mut count: usize = 0;

                for neigh in niter2 {
                    let edge = self.graph.find_edge(node, neigh).unwrap();
                    let ends = self.graph.edge_endpoints(edge).unwrap();
                    let t = thick[count];
                    let t2 = thick[count + 1];

                    let x1 = self.graph.node_weight(ends.0).unwrap().x;
                    let y1 = self.graph.node_weight(ends.0).unwrap().y;
                    let x2 = self.graph.node_weight(ends.1).unwrap().x;
                    let y2 = self.graph.node_weight(ends.1).unwrap().y;
                    let normal1 = normalize(x1, y1, x2, y2);
                    let normal2 = normalize((x1 + x2) / 2., (y1 + y2) / 2., x1, y1);
                    let d = ((t2 * t2 / 4.) - (t * t / 4.)).abs().sqrt();
                    count += 1;
                    /*                     println!("1({}, {}) 2({}, {})", x1, y1, x2, y2);
                     */
                    println!("{}", d);
                    points.append(&mut vec![
                        (
                            (normal2[1] - normal2[0]) * t,
                            (-normal2[0] - normal2[0]) * t,
                        ),
                        (
                            (-normal2[1] - normal2[0]) * t,
                            (normal2[0] - normal2[0]) * t,
                        ),
                        (
                            normal1[0] * d + normal1[1] * t / 2.,
                            normal1[1] * d - normal1[0] * t / 2.,
                        ),
                    ]);
                }

                Polygon::new(LineString::from(points), vec![])
            })
            .collect();

        /*         println!("{:?}", clusters);
         */
        let mut unionized: MultiPolygon<f32> = MultiPolygon(vec![
            polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),],
            polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),],
        ]);
        /*         let unionized: Polygon<f32> = polygon![(x: 0.,y: 0.),(x: 0.,y: 0.),];
         */
        for poly in clusters {
            unionized = poly.union(&unionized);
        }
        /*         let test = clusters[5].clone();
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
            }
            None => println!("No edges."),
        }
    }
    false
}
