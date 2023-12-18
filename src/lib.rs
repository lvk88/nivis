use wasm_bindgen::prelude::*;

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
pub struct GridData<'a>{
    grid: &'a Grid,
    data: Array2D
}

impl<'a> GridData<'a>{
    pub fn new(grid: &Grid, data: Array2D) -> GridData{
        GridData{
            grid,
            data
        }
    }

    pub fn new_with_function(grid: &Grid, init: impl Fn(f64, f64) -> f64) -> GridData{
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
            grid,
            data: array
        }
    }

    pub fn value(&self, i: usize, j: usize) -> f64{
        self.data.data[self.data.ravel(i,j)]
    }

    pub fn diff_x(&self) -> GridData{
        let mut data = vec![0.; self.grid.size[0] * self.grid.size[1]];
        (1..self.grid.size[1] - 1).cartesian_product(1..self.grid.size[0] - 1).for_each(|(j,i)|{
            let index = self.data.ravel(i,j);
            data[index] = (self.data.value(i + 1, j) - self.data.value(i - 1, j)) / (2. * self.grid.delta[0]);
        });

        let result_array = Array2D{
            data,
            size: self.grid.size
        };

        GridData{
            grid: self.grid,
            data: result_array
        }
    }

    pub fn diff_y(&self) -> GridData{
        let mut data = vec![0.; self.grid.size[0] * self.grid.size[1]];
        (1..self.grid.size[1] - 1).cartesian_product(1..self.grid.size[0] - 1).for_each(|(j,i)|{
            let index = self.data.ravel(i,j);
            data[index] = (self.data.value(i, j + 1) - self.data.value(i, j - 1)) / (2. * self.grid.delta[1]);
        });

        let result_array = Array2D{
            data,
            size: self.grid.size
        };

        GridData{
            grid: self.grid,
            data: result_array
        }
    }

    pub fn laplace(&self) -> GridData {
        let mut data = vec![0.; self.grid.size[0] * self.grid.size[1]];
        (1..self.grid.size[1] - 1).cartesian_product(1..self.grid.size[0] - 1).for_each(|(j,i)|{
            let index = self.data.ravel(i,j);

            let ddx = (self.data.value(i + 1, j) - 2. * self.data.value(i, j) + self.data.value(i - 1, j)) / (self.grid.delta[0] * self.grid.delta[0]);
            let ddy = (self.data.value(i, j + 1) - 2. * self.data.value(i, j) + self.data.value(i, j - 1)) / (self.grid.delta[1] * self.grid.delta[1]);


            data[index] = ddx + ddy;
        });

        let result_array = Array2D{
            data,
            size: self.grid.size
        };

        GridData{
            grid: self.grid,
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;

    #[test]
    fn create_a_grid() {
        let grid = Grid::new([0.1, 0.2], [3, 4]);
        let data = Array2D::new(grid.size);

        let grid_data = GridData::new(&grid, data);

        let data2 = Array2D::new(grid.size);
        let grid_data_2 = GridData::new(&grid, data2);

        assert_f64_near!(grid_data.data.data[0], 0.);
        assert_f64_near!(grid_data_2.data.data[0], 0.);
    }

    #[test]
    fn create_a_grid_with_function() {
        let grid = Grid::new([0.1, 0.2], [3, 4]);
        let grid_data = GridData::new_with_function(&grid, |x,y|{ x + y });

        assert_f64_near!(grid_data.value(0,0), 0.0);
        assert_f64_near!(grid_data.value(1,0), 0.1);
        assert_f64_near!(grid_data.value(0,1), 0.2);
        assert_f64_near!(grid_data.value(1,1), 0.3);
        assert_f64_near!(grid_data.value(2,3), 0.8);
    }

    #[test]
    fn test_first_derivatives() {
        let grid = Grid::new([0.1, 0.2], [3, 4]);
        let grid_data = GridData::new_with_function(&grid, |x,y|{ x * x + y * y });

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
        let grid = Grid::new([0.1, 0.2], [3, 4]);
        let grid_data = GridData::new_with_function(&grid, |x,y|{ x * x + y * y });

        let laplace = grid_data.laplace();

        println!("{:?}", grid_data.data.data);
        println!("{:?}", laplace.data.data);

        assert_f64_near!(laplace.value(0,0), 0.0);
        assert_f64_near!(laplace.value(1,0), 0.0);
        assert_f64_near!(laplace.value(1,1), 4.0);
    }
}
