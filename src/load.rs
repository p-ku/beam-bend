use flo_curves::{bezier, Coord2};

pub struct PointLoad {
    position: f32, // distance from left end of beam
    mag: f32,
}
pub enum Moment {
    Simple { left: f32, center: f32, right: f32 },
    Cantilever { left: f32, center: f32, right: f32 },
}

impl Moment {
    pub fn beam_curve(&self, left: f32, center: f32, right: f32) /* -> bezier::Curve  */
    {
        /*         match self {
            Moment::Simple {
                left,
                center,
                right,
            } => {
                if center != &0. {}
                if left == right && left != &0. {
                } else {
                    if left != &0. {
                                                let p = [
                            (0, 0),
                            (1 / 3, left / (9. * stiff)),
                            (2 / 3, (moment / (6. * stiff)) + (2 * moment) / (9 * stiff)),
                            (1, 0),
                        ];
                    }
                    if left != &0. {}
                }
            }
            Moment::Cantilever {
                left: 0.,
                center: 0.,
                right,
            } => if right != &0. {},
        } */
    }
}

/*         let p = [
    (0, 0),
    (1 / 3, moment / (9 * stiff)),
    (2 / 3, (moment / (6 * stiff)) + (2 * moment) / (9 * stiff)),
    (1, 0),
]; */
