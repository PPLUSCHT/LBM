use std::collections::HashSet;
use super::{Shape, curve::Curve};

pub struct CurveCollection{
    last_point: Option<(isize, isize)>,
    points: HashSet::<(isize, isize, bool)>
}

impl Shape for CurveCollection{
    fn get_points(&self) -> &HashSet<(isize, isize, bool)> {
        &self.points
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

}

impl CurveCollection{
    pub fn new() ->  CurveCollection{
        CurveCollection {last_point: None, points: HashSet::<(isize, isize, bool)>::new() }
    }

    pub fn add_curve(&mut self, curve: Curve){
        self.last_point = curve.last_point;
        self.points.extend(curve.points);
    }
}
