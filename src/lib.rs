use wasm_bindgen::prelude::*;

use itertools::Itertools;

// This is just a 2D matrix
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
}
