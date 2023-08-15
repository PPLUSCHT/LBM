use std::collections::HashSet;
use::line_drawing::Bresenham;
use web_sys::console;

use super::Shape;

pub struct Line{
    pub points: HashSet<(isize, isize, bool)>
}

impl Shape for Line{
    fn get_points(&self) -> &HashSet<(isize, isize, bool)> {
        &self.points
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
} 

impl Line{
    pub fn new(
        end_point_1: (isize, isize), 
        end_point_2: (isize, isize), 
        xdim: isize, 
        ydim: isize) -> Result<Line, String>{
            
            if !Self::validate(end_point_1, end_point_2, xdim, ydim){
                console::error_1(&format!("Endpoints ({},{}) ({},{}) are invalid with dimensions {} and {}", end_point_1.0, end_point_1.1,  end_point_2.0, end_point_2.1,  xdim, ydim).into());
                return Err(format!("Endpoints ({},{}) ({},{}) are invalid with dimensions {} and {}", end_point_1.0, end_point_1.1,  end_point_2.0, end_point_2.1,  xdim, ydim));
            }
            
            let mut points = HashSet::<(isize, isize, bool)>::new();

            for point in Self::generate_endpoints(end_point_1, end_point_2).iter(){
                if Self::validate(point.0, point.1, xdim, ydim){
                    let mut previous = point.0.clone();
                    for i in Bresenham::new(point.0,  point.1){
                        points.insert((i.0, i.1, true));
                        if Self::diagonal_step(previous,i){
                            let difference = (i.0 - previous.0, i.1 - previous.1);
                            points.insert((i.0 - difference.0, i.1, true));
                            points.insert((i.0, i.1 - difference.1, true));
                        }
                        previous = i;
                    }
                }
            }
            Ok(
                Line{
                    points
                }
            )
    }

    pub fn new_erased(
        end_point_1: (isize, isize), 
        end_point_2: (isize, isize), 
        xdim: isize, 
        ydim: isize) -> Result<Line, String>{
            if !Self::validate(end_point_1, end_point_2, xdim, ydim){
                console::log_1(&format!("Endpoints ({},{}) ({},{}) are invalid with dimensions {} and {}", end_point_1.0, end_point_1.1,  end_point_2.0, end_point_2.1,  xdim, ydim).into());
                return Err(format!("Endpoints ({},{}) ({},{}) are invalid with dimensions {} and {}", end_point_1.0, end_point_1.1,  end_point_2.0, end_point_2.1,  xdim, ydim));
            }
            
            let mut points = HashSet::<(isize, isize, bool)>::new();

            for point in Self::generate_endpoints_variable(end_point_1, end_point_2, 30).iter(){
                if Self::validate(point.0, point.1, xdim, ydim){
                    let mut previous = point.0.clone();
                    for i in Bresenham::new(point.0,  point.1){
                        points.insert((i.0, i.1, false));
                        if Self::diagonal_step(previous,i){
                            let difference = (i.0 - previous.0, i.1 - previous.1);
                            points.insert((i.0 - difference.0, i.1, false));
                            points.insert((i.0, i.1 - difference.1, false));
                        }
                        previous = i;
                    }
                }
            }
            Ok(
                Line{
                    points
                }
            )
    }

    fn diagonal_step(previous_point: (isize, isize),
                     next_point: (isize, isize)) -> bool{
        previous_point.0 - next_point.0 != 0 && previous_point.1 - next_point.1 != 0
    }

    fn generate_endpoints( end_point_1: (isize, isize), end_point_2: (isize, isize)) -> Vec<((isize, isize), (isize, isize))>{
        let mut output = Vec::<((isize, isize), (isize, isize))>::new();
        output.push(Self::order(end_point_1, end_point_2));

        let left_endpoints = if output[0].0.1 > output[0].1.1 {
                ((output[0].0.0, output[0].0.1 - 1),(output[0].0.0 + 1, output[0].0.1))
            }else{
                ((output[0].0.0, output[0].0.1 + 1),(output[0].0.0 + 1, output[0].0.1))
            };

        let right_endpoints = if output[0].0.1 > output[0].1.1 {
                ((output[0].1.0 - 1, output[0].1.1),(output[0].1.0, output[0].1.1 - 1))
            } else {
                ((output[0].1.0 - 1, output[0].1.1),(output[0].1.0, output[0].1.1 - 1))
            };
            
        output.push((left_endpoints.0, right_endpoints.0));
        output.push((left_endpoints.1, right_endpoints.1));

        output
    }

    fn generate_endpoints_variable( end_point_1: (isize, isize), end_point_2: (isize, isize), thickness: usize) -> Vec<((isize, isize), (isize, isize))>{
        let mut output = Vec::<((isize, isize), (isize, isize))>::new();
        output.push(Self::order(end_point_1, end_point_2));

        let mut left_endpoints = Vec::<((isize, isize),(isize, isize))>::new();
        let mut right_endpoints = Vec::<((isize, isize),(isize, isize))>::new();

        for i in 1..thickness{
            let i = i as isize;
            if output[0].0.1 > output[0].1.1 { 
                left_endpoints.push(((output[0].0.0, output[0].0.1 - i),(output[0].0.0 + i, output[0].0.1)));
                right_endpoints.push(((output[0].1.0 - i, output[0].1.1),(output[0].1.0, output[0].1.1 - i))); 
            } else {
                left_endpoints.push(((output[0].0.0, output[0].0.1 + i),(output[0].0.0 + i, output[0].0.1)));
                right_endpoints.push(((output[0].1.0 - i, output[0].1.1),(output[0].1.0, output[0].1.1 - i)));
            }
        }

        for i in 0..left_endpoints.len(){
            output.push((left_endpoints[i].0, right_endpoints[i].0));
            output.push((left_endpoints[i].1, right_endpoints[i].1));
        }

        output
    }

    fn order(point1: (isize, isize), point2: (isize, isize)) -> ((isize, isize),(isize, isize)){
        if point1.0 > point2.0{
            return (point1, point2);
        }
        (point2, point1)
    }

    pub fn join(&mut self, shape: Box<dyn Shape>){
        self.points.extend(shape.get_points())
    }

    fn validate(ep1: (isize, isize), ep2: (isize, isize), xdim: isize, ydim: isize) -> bool{
        ep1.0 >= 0 && ep2.0 >= 0 && ep1.1 >= 0 && ep2.1 >=0 && ep1.0 < xdim && ep2.0 < xdim && ep1.1 < ydim && ep2.1 < ydim
    }

}