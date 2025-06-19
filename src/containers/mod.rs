use std::ops::{Index, IndexMut};

pub mod graphical;

pub trait Container: Index<usize> {}

pub trait ContainerMut: IndexMut<usize> {}
