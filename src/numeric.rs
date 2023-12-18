use crate::array::{Array2D};
use itertools::Itertools;

pub fn diff_x(array: &Array2D, dx: f32) -> Array2D{
    let mut data = vec![0.; array.size[0] * array.size[1]];
    (1..array.size[1] - 1).cartesian_product(1..array.size[0] - 1).for_each(|(j,i)|{
        let index = array.ravel(i,j);
        data[index] = (array.value(i + 1, j) - array.value(i - 1, j)) / (2. * dx);
    });

    Array2D{
        data,
        ..*array
    }
}

pub fn diff_y(array: &Array2D, dy: f32) -> Array2D{
    let mut data = vec![0.; array.size[0] * array.size[1]];
    (1..array.size[1] - 1).cartesian_product(1..array.size[0] - 1).for_each(|(j,i)|{
        let index = array.ravel(i,j);
        data[index] = (array.value(i, j + 1) - array.value(i, j - 1)) / (2. * dy);
    });

    Array2D{
        data,
        ..*array
    }
}

pub fn laplace(array: &Array2D, dx: f32, dy: f32) -> Array2D{
    let mut data = vec![0.; array.size[0] * array.size[1]];

    (1..array.size[1] - 1).cartesian_product(1..array.size[0] - 1).for_each(|(j,i)|{
        let index = array.ravel(i,j);

        let ddx = (array.value(i + 1, j) - 2. * array.value(i, j) + array.value(i - 1, j)) / (dx * dx);
        let ddy = (array.value(i, j + 1) - 2. * array.value(i, j) + array.value(i, j - 1)) / (dy * dy);

        data[index] = ddx + ddy;
    });

    Array2D{
        data,
        ..*array
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;
    use crate::array::new_array_with_function;

    #[test]
    fn test_first_derivatives() {
        let grid_data = new_array_with_function(3, 4, 0.1, 0.2, |x,y|{ x * x + y * y});

        let dfdx = diff_x(&grid_data, 0.1);
        assert_f32_near!(dfdx.value(0,0), 0.0);
        assert_f32_near!(dfdx.value(1,0), 0.0);
        assert_f32_near!(dfdx.value(1,1), 0.2);

        let dfdy = diff_y(&grid_data, 0.2);
        assert_f32_near!(dfdy.value(0,0), 0.0);
        assert_f32_near!(dfdy.value(1,0), 0.0);
        assert_f32_near!(dfdy.value(1,1), 0.4);
    }

    #[test]
    fn test_laplace() {
        let grid_data = new_array_with_function(3, 4, 0.1, 0.2, |x,y|{ x * x + y * y});

        let laplace = laplace(&grid_data, 0.1, 0.2);

        assert_f32_near!(laplace.value(0,0), 0.0);
        assert_f32_near!(laplace.value(1,0), 0.0);
        assert_f32_near!(laplace.value(1,1), 4.0);
    }

}
