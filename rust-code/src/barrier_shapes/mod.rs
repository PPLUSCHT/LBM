use std::collections::HashSet;

use self::blob::Blob;

pub mod line;
pub mod merge_shapes;
pub mod blob;
pub mod curve;
pub mod curve_collection;

pub trait Shape {
    fn get_points(&self) -> &HashSet<(isize, isize, bool)>;
    fn is_empty(&self) -> bool;
    fn negate(&self) -> Blob {
        Blob{
            points: self.get_points().into_iter().map(|x| (x.0, x.1, !x.2)).collect()
        }
    }
}