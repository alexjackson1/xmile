use std::ops::{Index, IndexMut};

pub mod graphical;

pub trait Container: Index<usize> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn mean(&self) -> Option<f64>;
    fn min(&self) -> Option<f64>;
    fn max(&self) -> Option<f64>;
}

pub trait ContainerMut: Container + IndexMut<usize> {}
