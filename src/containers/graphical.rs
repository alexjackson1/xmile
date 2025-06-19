use std::ops::{Index, IndexMut};

use crate::{
    Identifier,
    containers::{Container, ContainerMut},
};

#[derive(Debug, Clone, PartialEq)]
pub enum GraphicalFunctionData {
    UniformScale {
        x_scale: (f64, f64),
        y_values: Vec<f64>,
    },
    XYPairs {
        x_values: Vec<f64>,
        y_values: Vec<f64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphicalFunctionType {
    Continuous,
    Extrapolate,
    Discrete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicalFunction {
    pub name: Identifier,
    pub data: GraphicalFunctionData,
    pub function_type: Option<GraphicalFunctionType>,
}

impl Index<usize> for GraphicalFunction {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.data {
            GraphicalFunctionData::UniformScale { y_values, .. } => &y_values[index],
            GraphicalFunctionData::XYPairs { y_values, .. } => &y_values[index],
        }
    }
}

impl IndexMut<usize> for GraphicalFunction {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match &mut self.data {
            GraphicalFunctionData::UniformScale { y_values, .. } => &mut y_values[index],
            GraphicalFunctionData::XYPairs { y_values, .. } => &mut y_values[index],
        }
    }
}

impl Container for GraphicalFunction {}

impl ContainerMut for GraphicalFunction {}
