use wasm_bindgen::prelude::*;

use std::ops::{Add, Sub};

use std::rc::{Rc, Weak};

use itertools::Itertools;

// This is just a 2D array
pub struct Array2D{
    size: [usize; 2],
    data: Vec<f64>
}


// This stores the geometry of a grid, i.e. the spacing between grid nodes
pub struct Grid{
    delta: [f64; 2],
    size: [usize; 2],
}

// This merges the geometry of a grid with associated data
pub struct GridData{
    grid: Weak<Grid>,
    data: Array2D
}

pub struct Simulation{
    grid: Rc<Grid>,
    temperature: GridData
    phi: GridData
}

impl Simulation{
    pub fn new(width: usize, height: usize) -> Simulation{
        let grid = Rc::new(Grid::new([0.03, 0.03], [width, height]));

        let temperature = GridData::new_with_function(Rc::downgrade(&grid), |_, _| 0.0);
        let phi = GridData::new_with_function(Rc::downgrade(&grid), |_, _| 0.0);

        Simulation{
            grid,
            temperature,
            phi
        }
    }
}

impl GridData{
    pub fn new(grid: Weak<Grid>, data: Array2D) -> GridData{
        GridData{
            grid,
            data
        }
    }

    pub fn new_with_function(grid: Weak<Grid>, init: impl Fn(f64, f64) -> f64) -> GridData{

        let grid = grid.upgrade().unwrap();

        let data = (0..grid.size[1]).cartesian_product(0..grid.size[0]).map(|(j, i)|{
            let x = i as f64 * grid.delta[0];
            let y = j as f64 * grid.delta[1];
            init(x,y)
        }).collect();

        let array = Array2D{
            size: grid.size,
            data
        };

        GridData{
            grid: Rc::downgrade(&grid),
            data: array
        }
    }

    pub fn value(&self, i: usize, j: usize) -> f64{
        self.data.data[self.data.ravel(i,j)]
    }

    pub fn diff_x(&self) -> GridData{
        let grid = self.grid.upgrade().unwrap();
        let mut data = vec![0.; grid.size[0] * grid.size[1]];
        (1..grid.size[1] - 1).cartesian_product(1..grid.size[0] - 1).for_each(|(j,i)|{
            let index = self.data.ravel(i,j);
            data[index] = (self.data.value(i + 1, j) - self.data.value(i - 1, j)) / (2. * grid.delta[0]);
        });

        let result_array = Array2D{
            data,
            size: grid.size
        };

        GridData{
            grid: self.grid.clone(),
            data: result_array
        }
    }

    pub fn diff_y(&self) -> GridData{
        let grid = self.grid.upgrade().unwrap();
        let mut data = vec![0.; grid.size[0] * grid.size[1]];
        (1..grid.size[1] - 1).cartesian_product(1..grid.size[0] - 1).for_each(|(j,i)|{
            let index = self.data.ravel(i,j);
            data[index] = (self.data.value(i, j + 1) - self.data.value(i, j - 1)) / (2. * grid.delta[1]);
        });

        let result_array = Array2D{
            data,
            size: grid.size
        };

        GridData{
            grid: self.grid.clone(),
            data: result_array
        }
    }

    pub fn laplace(&self) -> GridData {
        let grid = self.grid.upgrade().unwrap();
        let mut data = vec![0.; grid.size[0] * grid.size[1]];
        (1..grid.size[1] - 1).cartesian_product(1..grid.size[0] - 1).for_each(|(j,i)|{
            let index = self.data.ravel(i,j);

            let ddx = (self.data.value(i + 1, j) - 2. * self.data.value(i, j) + self.data.value(i - 1, j)) / (grid.delta[0] * grid.delta[0]);
            let ddy = (self.data.value(i, j + 1) - 2. * self.data.value(i, j) + self.data.value(i, j - 1)) / (grid.delta[1] * grid.delta[1]);


            data[index] = ddx + ddy;
        });

        let result_array = Array2D{
            data,
            size: grid.size
        };

        GridData{
            grid: self.grid.clone(),
            data: result_array
        }
    }
}

impl Grid{
    pub fn new(delta: [f64; 2], size: [usize; 2]) -> Grid{
        Grid{
            delta,
            size
        }
    }
}

impl Array2D{
    pub fn new(size: [usize; 2]) -> Array2D{
        let data = vec![0.; size[0] * size[1]];
        Array2D{
            size,
            data
        }
    }

    fn ravel(&self, i: usize, j: usize) -> usize{
        j * self.size[0] + i
    }

    fn value(&self, i: usize, j: usize) -> f64{
        self.data[self.ravel(i,j)]
    }
}

impl Add<Array2D> for Array2D{
    type Output = Self;

    fn add(self, rhs: Array2D) -> Array2D{
        let data = self.data.iter().zip(rhs.data.iter()).map(|(l, r)|{
            l + r
        }).collect();

        Array2D{
            data,
            ..self
        }
    }
}

impl Add<f64> for Array2D{
    type Output = Self;

    fn add(self, rhs: f64) -> Array2D{
        let data = self.data.iter().map(|lhs|{
           lhs + rhs
        }).collect();

        Array2D{
            data,
            ..self
        }
    }
}

impl Add<Array2D> for f64{
    type Output = Array2D;

    fn add(self, rhs: Array2D) -> Array2D{
        rhs + self
    }
}

fn atan2(y: &Array2D, x: &Array2D) -> Array2D{
    let data = y.data.iter().zip(x.data.iter()).map(|(y, x)|{
        y.atan2(*x)
    }).collect();

    Array2D{
        data,
        ..*y
    }
}

fn cos(x: &Array2D) -> Array2D{
    let data = x.data.iter().map(|x|{
        x.cos()
    }).collect();

    Array2D{
        data,
        ..*x
    }
}

fn sin(x: &Array2D) -> Array2D{
    let data = x.data.iter().map(|x|{
        x.sin()
    }).collect();

    Array2D{
        data,
        ..*x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;

    #[test]
    fn create_a_grid() {
        let grid = Rc::new(Grid::new([0.1, 0.2], [3, 4]));
        let data = Array2D::new(grid.size);

        let grid_data = GridData::new(Rc::downgrade(&grid), data);

        let data2 = Array2D::new(grid.size);
        let grid_data_2 = GridData::new(Rc::downgrade(&grid), data2);

        assert_f64_near!(grid_data.data.data[0], 0.);
        assert_f64_near!(grid_data_2.data.data[0], 0.);
    }

    #[test]
    fn create_a_grid_with_function() {
        let grid = Rc::new(Grid::new([0.1, 0.2], [3, 4]));
        let grid_data = GridData::new_with_function(Rc::downgrade(&grid), |x,y|{ x + y });

        assert_f64_near!(grid_data.value(0,0), 0.0);
        assert_f64_near!(grid_data.value(1,0), 0.1);
        assert_f64_near!(grid_data.value(0,1), 0.2);
        assert_f64_near!(grid_data.value(1,1), 0.3);
        assert_f64_near!(grid_data.value(2,3), 0.8);
    }

    #[test]
    fn test_first_derivatives() {
        let grid = Rc::new(Grid::new([0.1, 0.2], [3, 4]));
        let grid_data = GridData::new_with_function(Rc::downgrade(&grid), |x,y|{ x * x + y * y});

        let dfdx = grid_data.diff_x();
        assert_f64_near!(dfdx.value(0,0), 0.0);
        assert_f64_near!(dfdx.value(1,0), 0.0);
        assert_f64_near!(dfdx.value(1,1), 0.2);

        let dfdy = grid_data.diff_y();
        assert_f64_near!(dfdy.value(0,0), 0.0);
        assert_f64_near!(dfdy.value(1,0), 0.0);
        assert_f64_near!(dfdy.value(1,1), 0.4);
    }

    #[test]
    fn test_laplace() {
        let grid = Rc::new(Grid::new([0.1, 0.2], [3, 4]));
        let grid_data = GridData::new_with_function(Rc::downgrade(&grid), |x,y|{ x * x + y * y });

        let laplace = grid_data.laplace();

        println!("{:?}", grid_data.data.data);
        println!("{:?}", laplace.data.data);

        assert_f64_near!(laplace.value(0,0), 0.0);
        assert_f64_near!(laplace.value(1,0), 0.0);
        assert_f64_near!(laplace.value(1,1), 4.0);
    }

    #[test]
    fn test_atan2_positive_values() {
        let y = Array2D {
            size: [1, 3],
            data: vec![1.0, 2.0, 3.0],
        };

        let x = Array2D {
            size: [1, 3],
            data: vec![4.0, 5.0, 6.0],
        };

        let result = atan2(&y, &x);

        let expected = Array2D {
            size: [1, 3],
            data: vec![0.24497866312686414, 0.3805063771123649, 0.4636476090008061],
        };

        assert_eq!(result.size, expected.size);
        assert_eq!(result.data.len(), expected.data.len());
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], expected.data[i], "Mismatch at index {}", i);
        }
    }

    #[test]
    fn test_atan2_negative_values() {
        let y = Array2D {
            size: [1, 3],
            data: vec![-1.0, -2.0, -3.0],
        };

        let x = Array2D {
            size: [1, 3],
            data: vec![-4.0, -5.0, -6.0],
        };

        let result = atan2(&y, &x);

        let expected = Array2D {
            size: [1, 3],
            data: vec![-2.896613990462929, -2.761086276477428, -2.677945044588987],
        };

        assert_eq!(result.size, expected.size);
        assert_eq!(result.data.len(), expected.data.len());
        for i in 0..result.data.len() {
            assert_eq!(result.data[i], expected.data[i], "Mismatch at index {}", i);
        }
    }

    #[test]
    fn test_add_arrays() {
        let x = Array2D{
            size: [1, 3],
            data: vec![1.0, 2.0, 3.0]
        };

        let y = Array2D{
            size: [1, 3],
            data: vec![4.0, 5.0, 6.0]
        };

        let result = x + y;

        assert_eq!(result.data[0], 5.0);
        assert_eq!(result.data[1], 7.0);
        assert_eq!(result.data[2], 9.0);
    }

    #[test]
    fn test_add_scalar_toarray() {
        let x = Array2D{
            size: [1, 3],
            data: vec![1.0, 2.0, 3.0]
        };

        let result = x + 5.;

        assert_eq!(result.data[0], 6.0);
        assert_eq!(result.data[1], 7.0);
        assert_eq!(result.data[2], 8.0);

        let x = Array2D{
            size: [1, 3],
            data: vec![1.0, 2.0, 3.0]
        };

        let result = x + 6.;

        assert_eq!(result.data[0], 7.0);
        assert_eq!(result.data[1], 8.0);
        assert_eq!(result.data[2], 9.0);
    }

    #[test]
    fn test_sin_cos_on_array(){
        let x = Array2D{
            size: [1, 3],
            data: vec![0.0, std::f64::consts::PI / 4., std::f64::consts::PI / 2.0]
        };

        let result = sin(&x);

        assert_f64_near!(result.data[0], 0.0);
        assert_f64_near!(result.data[1], std::f64::consts::FRAC_1_SQRT_2);
        assert_f64_near!(result.data[2], 1.0);

        let result = cos(&x);
        assert_f64_near!(result.data[0], 1.0);
        assert_f64_near!(result.data[1], std::f64::consts::FRAC_1_SQRT_2);
        // TODO: understand how to do this assertion
        //assert_f64_near!(result.data[2], 0.0, 100000);
    }
}
