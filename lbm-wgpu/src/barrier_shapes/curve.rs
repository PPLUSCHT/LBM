use std::collections::HashSet;
use super::{Shape, line::Line};

pub struct Curve{
    pub points: HashSet<(isize, isize, bool)>,
    pub last_point: Option<(isize, isize)>,
}

impl Shape for Curve{
    fn get_points(&self) -> &HashSet<(isize, isize, bool)>{
        &self.points
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl Curve{

    pub fn new() -> Curve{
        Curve { 
            points: HashSet::<(isize, isize, bool)>::new(), 
            last_point: None,
        }
    }

    pub fn add_segment(&mut self, next: (isize, isize), xdim: isize, ydim: isize){
        if self.last_point != None{
            self.points.extend(&Line::new(self.last_point.unwrap().clone(), next, xdim, ydim).unwrap().points);
            self.last_point = Some(next);
        }
        else {
            self.points.insert((next.0, next.1, true));
            self.last_point = Some(next);
        }
    }

    pub fn erase_segment(&mut self, next: (isize, isize), xdim: isize, ydim: isize){
        if self.last_point != None{
            self.points.extend(&Line::new_erased(self.last_point.unwrap().clone(), next, xdim, ydim).unwrap().points);
            self.last_point = Some(next);
        }
        else {
            self.points.insert((next.0, next.1, false));
            self.last_point = Some(next);
        }
    }

    pub fn empty(&mut self){
        self.points.clear();
        self.last_point = None;
    }

    pub fn join(&mut self, shape: Box<dyn Shape>){
        self.points.extend(shape.get_points())
    }
}