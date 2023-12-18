use wasm_bindgen::prelude::*;

pub struct Array2D{
    size: [usize; 2],
    data: Vec<f64>
}

pub struct Grid{
    delta: [f64; 2],
    size: [usize; 2],
}

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

        assert_f64_near!(grid_data.data.data[0], 0.);
    }
}
