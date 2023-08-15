use std::collections::HashSet;


use super::Shape;
pub struct Blob{
    pub points: HashSet<(isize, isize, bool)>
}

impl Shape for Blob{
    fn get_points(&self) -> &HashSet<(isize, isize, bool)> {
        &self.points
    }
    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl Blob{
    pub fn new(points: HashSet<(isize, isize, bool)>) -> Blob{
        Blob{
            points
        }
    }

    pub fn new_empty() -> Blob{
        Blob { points: HashSet::<(isize, isize, bool)>::new() }
    }

    pub fn add(&mut self, points: &Vec<(isize, isize, bool)>, xdim: u32, ydim: u32){
        for p in points{
            if (p.0 as u32) < xdim && (p.1 as u32) < ydim{
                self.points.remove(&(p.0, p.1, !p.2));
                self.points.insert(p.clone());
            }
        }
    }

    pub fn join(&mut self, shape: &dyn Shape){
        for p in shape.get_points(){
            self.points.remove(&(p.0, p.1, !p.2));
            self.points.insert(p.clone());
        }
    }

    pub fn empty(&mut self){
        self.points.clear();
    }
}