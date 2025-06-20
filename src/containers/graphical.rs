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

impl GraphicalFunctionData {
    pub fn y_values(&self) -> &[f64] {
        match self {
            GraphicalFunctionData::UniformScale { y_values, .. } => y_values,
            GraphicalFunctionData::XYPairs { y_values, .. } => y_values,
        }
    }

    pub fn len(&self) -> usize {
        self.y_values().len()
    }

    pub fn mean(&self) -> Option<f64> {
        match self.y_values() {
            [] => None,
            y => Some(y.iter().sum::<f64>() / y.len() as f64),
        }
    }

    pub fn min(&self) -> Option<f64> {
        match self.y_values() {
            [] => None,
            y => Some(*y.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()),
        }
    }

    pub fn max(&self) -> Option<f64> {
        match self.y_values() {
            [] => None,
            y => Some(*y.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()),
        }
    }
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

impl Container for GraphicalFunction {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn mean(&self) -> Option<f64> {
        self.data.mean()
    }

    fn min(&self) -> Option<f64> {
        self.data.min()
    }

    fn max(&self) -> Option<f64> {
        self.data.max()
    }
}

impl ContainerMut for GraphicalFunction {}
