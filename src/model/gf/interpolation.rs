use super::{GraphicalFunctionData, GraphicalFunctionScale, Points};

impl GraphicalFunctionData {
    fn gradient(&self, i_1: usize, i_2: usize) -> Option<f64> {
        match self {
            GraphicalFunctionData::UniformScale {
                y_values, x_scale, ..
            } => {
                if y_values.len() < 2 {
                    return None;
                }

                let step_size = x_scale.delta() / (y_values.len() - 1) as f64;
                if (step_size - 0.0).abs() < f64::EPSILON {
                    return None;
                }

                Some((y_values[i_2] - y_values[i_1]) / step_size)
            }
            GraphicalFunctionData::XYPairs {
                x_values, y_values, ..
            } => {
                if x_values.len() < 2 {
                    return None;
                }

                Some((y_values[i_2] - y_values[i_1]) / (x_values[i_2] - x_values[i_1]))
            }
        }
    }

    fn left_gradient(&self) -> Option<f64> {
        if self.len() < 2 {
            return None;
        }
        self.gradient(0, 1)
    }

    fn right_gradient(&self) -> Option<f64> {
        if self.len() < 2 {
            return None;
        }
        self.gradient(self.len() - 2, self.len() - 1)
    }

    pub fn step_uniform(&self, x: f64, x_scale: &GraphicalFunctionScale, y_values: &Points) -> f64 {
        if x <= x_scale.min {
            return y_values[0];
        }

        if x >= x_scale.max {
            return y_values[y_values.len() - 1];
        }

        let step = x_scale.delta() / (y_values.len() - 1) as f64;
        if (step - 0.0).abs() < f64::EPSILON {
            return y_values[0];
        }

        let index = ((x - x_scale.min) / step).floor() as usize;
        y_values[index.min(y_values.len() - 1)]
    }

    pub fn step_xy(&self, x: f64, x_values: &Points, y_values: &Points) -> f64 {
        if x <= x_values[0] {
            return y_values[0];
        }

        if x >= x_values[x_values.len() - 1] {
            return y_values[y_values.len() - 1];
        }

        // Find the largest x-value <= x (next lower x-coordinate)
        let mut index = 0;
        for i in 0..x_values.len() {
            if x_values[i] <= x {
                index = i;
            } else {
                break;
            }
        }

        y_values[index]
    }

    pub fn interpolate_uniform(
        &self,
        x: f64,
        x_scale: &GraphicalFunctionScale,
        y_values: &Points,
        extrapolate: bool,
    ) -> f64 {
        assert!(!y_values.is_empty(), "y-values cannot be empty");

        let b_extrap = extrapolate as u8 as f64;

        if x < x_scale.min {
            let grad = self.left_gradient().unwrap_or_default();
            return y_values[0] + b_extrap * grad * (x - x_scale.min);
        }

        if x > x_scale.max {
            let grad = self.right_gradient().unwrap_or_default();
            return y_values[y_values.len() - 1] + b_extrap * grad * (x - x_scale.max);
        }

        if x_scale.delta().abs() < f64::EPSILON {
            return y_values[0];
        }

        let step = x_scale.delta() / (y_values.len() - 1) as f64;
        if (step - 0.0).abs() < f64::EPSILON {
            return y_values[0];
        }

        let index = ((x - x_scale.min) / step).floor() as usize;

        let lower_index = index.min(y_values.len() - 2);
        let upper_index = (lower_index + 1).min(y_values.len() - 1);

        if lower_index == upper_index {
            return y_values[lower_index];
        }

        let lower_x = x_scale.min + lower_index as f64 * step;
        let upper_x = x_scale.min + upper_index as f64 * step;

        let lower_y = y_values[lower_index];
        let upper_y = y_values[upper_index];

        let t = (x - lower_x) / (upper_x - lower_x);

        lower_y + t * (upper_y - lower_y)
    }

    pub fn interpolate_xy(
        &self,
        x: f64,
        x_values: &Points,
        y_values: &Points,
        extrapolate: bool,
    ) -> f64 {
        assert!(
            !x_values.is_empty() && !y_values.is_empty(),
            "x-values and y-values cannot be empty"
        );

        let b_extrap = extrapolate as u8 as f64;

        if x < x_values[0] {
            if x_values.len() < 2 {
                return y_values[0];
            }

            let gradient = (y_values[1] - y_values[0]) / (x_values[1] - x_values[0]);
            return y_values[0] + b_extrap * gradient * (x - x_values[0]);
        }

        let x_last = x_values[x_values.len() - 1];
        if x > x_last {
            if x_values.len() < 2 {
                return y_values[y_values.len() - 1];
            }

            let step_size = x_last - x_values[x_values.len() - 2];
            if (step_size - 0.0).abs() < f64::EPSILON {
                return y_values[y_values.len() - 1];
            }

            let gradient =
                (y_values[y_values.len() - 1] - y_values[y_values.len() - 2]) / step_size;
            return y_values[y_values.len() - 1] + b_extrap * gradient * (x - x_last);
        }

        let mut lower_index = 0;
        let mut upper_index = 0;
        for i in 0..x_values.len() {
            if x_values[i] <= x {
                lower_index = i;
            } else {
                upper_index = i;
                break;
            }
        }

        if lower_index == upper_index {
            return y_values[lower_index];
        }

        if upper_index >= x_values.len() {
            if x_values.len() < 2 {
                return y_values[y_values.len() - 1];
            }

            let step_size = x_last - x_values[x_values.len() - 2];
            if (step_size - 0.0).abs() < f64::EPSILON {
                return y_values[y_values.len() - 1];
            }

            let gradient =
                (y_values[y_values.len() - 1] - y_values[y_values.len() - 2]) / step_size;
            return y_values[y_values.len() - 1] + gradient * (x - x_last);
        }

        let lower_x = x_values[lower_index];
        let upper_x = x_values[upper_index];

        let lower_y = y_values[lower_index];
        let upper_y = y_values[upper_index];

        let t = (x - lower_x) / (upper_x - lower_x);
        lower_y + t * (upper_y - lower_y)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        GraphicalFunction, GraphicalFunctionType, Identifier, test_utils::assert_float_eq,
    };

    use super::*;

    // Test data from XMILE spec example
    fn create_food_availability_function() -> GraphicalFunction {
        GraphicalFunction {
            name: Some(Identifier::parse_default("food_availability_multiplier_function").unwrap()),
            data: GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.0, 0.3, 0.55, 0.7, 0.83, 0.9, 0.95, 0.98, 0.99, 0.995, 1.0],
                None,
            ),
            function_type: Some(GraphicalFunctionType::Continuous),
        }
    }

    #[test]
    fn test_continuous_interpolation_uniform_scale() {
        let gf = create_food_availability_function();

        // Test exact values
        assert_float_eq(gf.evaluate(0.0), 0.0, 1e-10);
        assert_float_eq(gf.evaluate(0.1), 0.3, 1e-10);
        assert_float_eq(gf.evaluate(1.0), 1.0, 1e-10);

        // Test interpolation between points
        // Between x=0.0 and x=0.1: y should be between 0.0 and 0.3
        let mid_value = gf.evaluate(0.05);
        assert!(mid_value > 0.0 && mid_value < 0.3);
        assert_float_eq(mid_value, 0.15, 1e-10); // Linear interpolation: 0.0 + 0.5 * (0.3 - 0.0)
    }

    #[test]
    fn test_continuous_clamping() {
        let gf = create_food_availability_function();

        // Test out-of-range values are clamped
        assert_float_eq(gf.evaluate(-0.5), 0.0, 1e-10); // Clamped to first value
        assert_float_eq(gf.evaluate(1.5), 1.0, 1e-10); // Clamped to last value
    }

    #[test]
    fn test_extrapolate_interpolation() {
        let mut gf = create_food_availability_function();
        gf.function_type = Some(GraphicalFunctionType::Extrapolate);

        // Test exact values (should be same as continuous)
        assert_float_eq(gf.evaluate(0.0), 0.0, 1e-10);
        assert_float_eq(gf.evaluate(1.0), 1.0, 1e-10);

        // Test extrapolation beyond range
        let extrapolated_low = gf.evaluate(-0.1);
        assert!(extrapolated_low < 0.0); // Should extrapolate below 0

        let extrapolated_high = gf.evaluate(1.1);
        assert!(extrapolated_high > 1.0); // Should extrapolate above 1
    }

    #[test]
    fn test_discrete_interpolation() {
        let gf = GraphicalFunction {
            name: None,
            data: GraphicalFunctionData::uniform_scale(
                (0.0, 1.0),
                vec![0.0, 0.5, 0.8, 0.8], // Last two values same for discrete
                None,
            ),
            function_type: Some(GraphicalFunctionType::Discrete),
        };

        // Test step function behaviour
        assert_float_eq(gf.evaluate(0.0), 0.0, 1e-10);
        assert_float_eq(gf.evaluate(0.2), 0.0, 1e-10); // Should stay at first value
        assert_float_eq(gf.evaluate(0.4), 0.5, 1e-10); // Jump to second value
        assert_float_eq(gf.evaluate(0.8), 0.8, 1e-10); // Jump to third value
        assert_float_eq(gf.evaluate(1.0), 0.8, 1e-10); // Same as last value
    }

    #[test]
    fn test_xy_pairs_interpolation() {
        let gf = GraphicalFunction {
            name: None,
            data: GraphicalFunctionData::xy_pairs(
                vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5],
                vec![0.05, 0.1, 0.2, 0.25, 0.3, 0.33],
                None,
            ),
            function_type: Some(GraphicalFunctionType::Continuous),
        };

        // Test exact values
        assert_float_eq(gf.evaluate(0.0), 0.05, 1e-10);
        assert_float_eq(gf.evaluate(0.1), 0.1, 1e-10);
        assert_float_eq(gf.evaluate(0.5), 0.33, 1e-10);

        // Test interpolation
        let interpolated = gf.evaluate(0.05);
        assert!(interpolated > 0.05 && interpolated < 0.1);
    }

    #[test]
    fn test_single_point_function() {
        let gf = GraphicalFunction {
            name: None,
            data: GraphicalFunctionData::uniform_scale(
                (0.0, 0.0), // Single point
                vec![0.5],
                None,
            ),
            function_type: Some(GraphicalFunctionType::Continuous),
        };

        // All evaluations should return the single y-value
        assert_float_eq(gf.evaluate(-1.0), 0.5, 1e-10);
        assert_float_eq(gf.evaluate(0.0), 0.5, 1e-10);
        assert_float_eq(gf.evaluate(1.0), 0.5, 1e-10);
    }
}
