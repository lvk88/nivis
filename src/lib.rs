use wasm_bindgen::prelude::*;

use web_sys::console;

use std::ops::{Add, Sub};

use std::rc::{Rc, Weak};

use itertools::Itertools;

// This is just a 2D array
pub struct Array2D{
    size: [usize; 2],
    data: Vec<f32>
}


// This stores the geometry of a grid, i.e. the spacing between grid nodes
pub struct Grid{
    delta: [f32; 2],
    size: [usize; 2],
}

// This merges the geometry of a grid with associated data
pub struct GridData{
    grid: Weak<Grid>,
    data: Array2D
}

#[wasm_bindgen]
pub struct Simulation{
    pub width: usize,
    pub height: usize,
    dx: f32,
    dy: f32,
    grid: Rc<Grid>,
    pub kappa: f32,
    pub delta: f32,
    temperature: GridData,
    phi: GridData
}

#[repr(C)]
struct RGBA{
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

const COLORMAP_BLUE_TO_WHITE: [RGBA; 10] = [
    RGBA{r: 30,  g: 65,  b: 101, a: 255},
    RGBA{r: 52,  g: 111, b: 171, a: 255},
    RGBA{r: 94,  g: 141, b: 188, a: 255},
    RGBA{r: 126, g: 163, b: 201, a: 255},
    RGBA{r: 152, g: 182, b: 212, a: 255},
    RGBA{r: 175, g: 198, b: 222, a: 255},
    RGBA{r: 196, g: 213, b: 230, a: 255},
    RGBA{r: 214, g: 226, b: 238, a: 255},
    RGBA{r: 232, g: 238, b: 245, a: 255},
    RGBA{r: 247, g: 250, b: 252, a: 255}
];

const COLORMAP_COOL_TO_WARM: [RGBA; 10] = [
    RGBA{r: 58,  g: 76,  b: 192, a: 255},
    RGBA{r: 88,  g: 118, b: 226, a: 255},
    RGBA{r: 123, g: 158, b: 248, a: 255},
    RGBA{r: 157, g: 189, b: 254, a: 255},
    RGBA{r: 192, g: 211, b: 245, a: 255},
    RGBA{r: 221, g: 220, b: 219, a: 255},
    RGBA{r: 241, g: 202, b: 182, a: 255},
    RGBA{r: 246, g: 171, b: 141, a: 255},
    RGBA{r: 237, g: 132, b: 103, a: 255},
    RGBA{r: 214, g: 82,  b: 67 , a: 255}
];

#[wasm_bindgen]
impl Simulation{

    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Simulation{
        let dx = 0.03;
        let dy = 0.03;
        let grid = Rc::new(Grid::new([dx, dy], [width + 2, height + 2]));

        let temperature = GridData::new_with_function(Rc::downgrade(&grid), |_, _| 0.0);

        let cx = (width / 2) as i64;
        let cy = (height / 2) as i64;
        let radius = 5;

        let kappa = 1.6;
        let delta = 0.04;

        let phi = GridData::new_with_function(Rc::downgrade(&grid), |x, y|{
            let i = (x / dx).floor() as i64;
            let j = (y / dy).floor() as i64;
            if (i - cx) * (i - cx) + (j - cy) * (j - cy) < radius * radius{
                1.0
            }else{
                0.0
            }
        });

        Simulation{
            width,
            height,
            dx,
            dy,
            kappa,
            delta,
            grid,
            temperature,
            phi
        }
    }

    pub fn get_temperature_rgb(&self) -> Vec<u8>{
        (0..self.height).cartesian_product(0..self.width).map(|(j,i)|{
            let value = self.temperature.value(i + 1, j + 1);
            let clamped_val = value.clamp(0.0, 1.0);
            let index = (clamped_val * COLORMAP_COOL_TO_WARM.len() as f32 - 1.0) as usize;
            let start_color = &COLORMAP_COOL_TO_WARM[index];
            [start_color.r, start_color.g, start_color.b, start_color.a]
        }).flatten().collect()
    }

    pub fn get_phi_rgb(&self) -> Vec<u8>{
        (0..self.height).cartesian_product(0..self.width).map(|(j,i)|{
            let value = self.phi.value(i + 1, j + 1);
            let clamped_val = value.clamp(0.0, 1.0);
            let index = (clamped_val * COLORMAP_COOL_TO_WARM.len() as f32 - 1.0) as usize;
            let start_color = &COLORMAP_BLUE_TO_WHITE[index];
            [start_color.r, start_color.g, start_color.b, start_color.a]
        }).flatten().collect()
    }

    pub fn step(&mut self){
        let delta_t = 1e-4;
        let epsilonb = 0.01;
        let aniso = 6.0;
        let theta0 = 0.0;
        let alpha = 0.9;
        let gamma = 10.0;
        let teq = 1.0;
        let tau = 0.0003;

        let dphidx = self.phi.diff_x();
        let dphidy = self.phi.diff_y();

        let laplace_phi = self.phi.laplace();
        let laplace_temperature = self.temperature.laplace();

        let theta = atan2(&dphidy.data, &dphidx.data);

        let aniso_x_theta_theta0: Vec<f32> = theta.data.iter().map(|val|{
            aniso * (val - theta0)
        }).collect();

        let epsilon: Vec<f32> = aniso_x_theta_theta0.iter().map(|val|{
            epsilonb * (1. + self.delta * val.cos())
        }).collect();

        let depsilondtheta: Vec<f32> = aniso_x_theta_theta0.iter().map(|val|{
            -epsilonb * aniso * self.delta * val.sin()
        }).collect();

        let epsilon_x_depsilondtheta: Vec<f32> = epsilon.iter().zip(depsilondtheta.iter()).map(|(eps, deps)|{
            eps * deps
        }).collect();

        let term1: Vec<f32> = epsilon_x_depsilondtheta.iter().zip(dphidx.data.data.iter()).map(|(lhs, rhs)|{
            lhs * rhs
        }).collect();

        let term1 = Array2D{
            data: term1,
            size: self.temperature.data.size.clone()
        };

        let term1 = GridData{
            data: term1,
            grid: dphidx.grid.clone()
        };
        let term1 = term1.diff_y();

        let term2: Vec<f32> = epsilon_x_depsilondtheta.iter().zip(dphidy.data.data.iter()).map(|(lhs, rhs)|{
            lhs * rhs
        }).collect();

        let term2 = Array2D{
            data: term2,
            size: self.temperature.data.size.clone()
        };

        let term2 = GridData{
            data: term2,
            grid: dphidx.grid.clone()
        };

        let term2 = term2.diff_x();

        let m: Vec<f32> = self.temperature.data.data.iter().map(|val|{
            alpha / std::f32::consts::PI * (gamma * (teq - val)).atan()
        }).collect();

        let new_phi: Vec<f32> = (0..self.phi.data.data.len()).map(|i|{
            self.phi.data.data[i] + (delta_t / tau) * (term1.data.data[i] - term2.data.data[i] + epsilon[i] * epsilon[i] * laplace_phi.data.data[i] + self.phi.data.data[i] * (1.0 - self.phi.data.data[i]) * (self.phi.data.data[i] - 0.5 + m[i]))
        }).collect();

        let new_temperature: Vec<f32> = (0..self.temperature.data.data.len()).map(|i|{
            self.temperature.data.data[i] + delta_t * laplace_temperature.data.data[i] + self.kappa * (new_phi[i] - self.phi.data.data[i])
        }).collect();

        self.temperature.data.data = new_temperature;
        self.phi.data.data = new_phi;
    }

    pub fn reset(&mut self){
        let temperature = GridData::new_with_function(Rc::downgrade(&self.grid), |_, _| 0.0);

        let cx = (self.width / 2) as i64;
        let cy = (self.height / 2) as i64;
        let radius = 5;

        let phi = GridData::new_with_function(Rc::downgrade(&self.grid), |x, y|{
            let i = (x / self.dx).floor() as i64;
            let j = (y / self.dy).floor() as i64;
            if (i - cx) * (i - cx) + (j - cy) * (j - cy) < radius * radius{
                1.0
            }else{
                0.0
            }
        });

        self.temperature = temperature;
        self.phi = phi;

    }

    pub fn add_seed(&mut self, x: i32, y: i32){
        for j in -2..2{
            if (y + j) < 0 || (y + j) as usize > self.height{
                continue;
            }
            for i in -2..2{
                if (x + j) < 0 || (x + i) as usize > self.width{
                    continue;
                }
                let cx = (x + i) as usize;
                let cy = (y + j) as usize;
                let index = self.phi.data.ravel(cx, cy);
                self.phi.data.data[index] = 1.0;
            }
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

    pub fn new_with_function(grid: Weak<Grid>, init: impl Fn(f32, f32) -> f32) -> GridData{

        let grid = grid.upgrade().unwrap();

        let data = (0..grid.size[1]).cartesian_product(0..grid.size[0]).map(|(j, i)|{
            let x = i as f32 * grid.delta[0];
            let y = j as f32 * grid.delta[1];
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

    pub fn value(&self, i: usize, j: usize) -> f32{
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

    pub fn diff_all(&self) -> (GridData, GridData, GridData){
        let grid = self.grid.upgrade().unwrap();

        let mut dfdx = vec![0.; grid.size[0] * grid.size[1]];
        let mut dfdy = vec![0.; grid.size[0] * grid.size[1]];
        let mut lap  = vec![0.; grid.size[0] * grid.size[1]];

        (1..grid.size[1] - 1).cartesian_product(1..grid.size[0] - 1).for_each(|(j,i)|{
            let index = self.data.ravel(i,j);

            dfdx[index] = (self.data.value(i + 1, j) - self.data.value(i - 1, j)) / (2. * grid.delta[0]);
            dfdy[index] = (self.data.value(i, j + 1) - self.data.value(i, j - 1)) / (2. * grid.delta[1]);

            let ddx = (self.data.value(i + 1, j) - 2. * self.data.value(i, j) + self.data.value(i - 1, j)) / (grid.delta[0] * grid.delta[0]);
            let ddy = (self.data.value(i, j + 1) - 2. * self.data.value(i, j) + self.data.value(i, j - 1)) / (grid.delta[1] * grid.delta[1]);


            lap[index] = ddx + ddy;
        });

        (
            GridData{
                grid: Rc::downgrade(&grid),
                data: Array2D{data: dfdx, size: grid.size}
            },
            GridData{
                grid: Rc::downgrade(&grid),
                data: Array2D{data: dfdy, size: grid.size}
            },
            GridData{
                grid: Rc::downgrade(&grid),
                data: Array2D{data: lap, size: grid.size}
            },
        )
    }
}

impl Grid{
    pub fn new(delta: [f32; 2], size: [usize; 2]) -> Grid{
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

    fn value(&self, i: usize, j: usize) -> f32{
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

impl Add<f32> for Array2D{
    type Output = Self;

    fn add(self, rhs: f32) -> Array2D{
        let data = self.data.iter().map(|lhs|{
           lhs + rhs
        }).collect();

        Array2D{
            data,
            ..self
        }
    }
}

impl Add<Array2D> for f32{
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

        assert_f32_near!(grid_data.data.data[0], 0.);
        assert_f32_near!(grid_data_2.data.data[0], 0.);
    }

    #[test]
    fn create_a_grid_with_function() {
        let grid = Rc::new(Grid::new([0.1, 0.2], [3, 4]));
        let grid_data = GridData::new_with_function(Rc::downgrade(&grid), |x,y|{ x + y });

        assert_f32_near!(grid_data.value(0,0), 0.0);
        assert_f32_near!(grid_data.value(1,0), 0.1);
        assert_f32_near!(grid_data.value(0,1), 0.2);
        assert_f32_near!(grid_data.value(1,1), 0.3);
        assert_f32_near!(grid_data.value(2,3), 0.8);
    }

    #[test]
    fn test_first_derivatives() {
        let grid = Rc::new(Grid::new([0.1, 0.2], [3, 4]));
        let grid_data = GridData::new_with_function(Rc::downgrade(&grid), |x,y|{ x * x + y * y});

        let dfdx = grid_data.diff_x();
        assert_f32_near!(dfdx.value(0,0), 0.0);
        assert_f32_near!(dfdx.value(1,0), 0.0);
        assert_f32_near!(dfdx.value(1,1), 0.2);

        let dfdy = grid_data.diff_y();
        assert_f32_near!(dfdy.value(0,0), 0.0);
        assert_f32_near!(dfdy.value(1,0), 0.0);
        assert_f32_near!(dfdy.value(1,1), 0.4);
    }

    #[test]
    fn test_laplace() {
        let grid = Rc::new(Grid::new([0.1, 0.2], [3, 4]));
        let grid_data = GridData::new_with_function(Rc::downgrade(&grid), |x,y|{ x * x + y * y });

        let laplace = grid_data.laplace();

        println!("{:?}", grid_data.data.data);
        println!("{:?}", laplace.data.data);

        assert_f32_near!(laplace.value(0,0), 0.0);
        assert_f32_near!(laplace.value(1,0), 0.0);
        assert_f32_near!(laplace.value(1,1), 4.0);
    }

    #[test]
    fn diff_all() {
        let grid = Rc::new(Grid::new([0.1, 0.2], [3, 4]));
        let grid_data = GridData::new_with_function(Rc::downgrade(&grid), |x,y|{ x * x + y * y });

        let (dfdx, dfdy, laplace) = grid_data.diff_all();

        assert_f32_near!(dfdx.value(0,0), 0.0);
        assert_f32_near!(dfdx.value(1,0), 0.0);
        assert_f32_near!(dfdx.value(1,1), 0.2);

        assert_f32_near!(dfdy.value(0,0), 0.0);
        assert_f32_near!(dfdy.value(1,0), 0.0);
        assert_f32_near!(dfdy.value(1,1), 0.4);

        assert_f32_near!(laplace.value(0,0), 0.0);
        assert_f32_near!(laplace.value(1,0), 0.0);
        assert_f32_near!(laplace.value(1,1), 4.0);
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
            data: vec![0.24497867, 0.3805064, 0.46364760],
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
            data: vec![0.0, std::f32::consts::PI / 4., std::f32::consts::PI / 2.0]
        };

        let result = sin(&x);

        assert_f32_near!(result.data[0], 0.0);
        assert_f32_near!(result.data[1], std::f32::consts::FRAC_1_SQRT_2);
        assert_f32_near!(result.data[2], 1.0);

        let result = cos(&x);
        assert_f32_near!(result.data[0], 1.0);
        assert_f32_near!(result.data[1], std::f32::consts::FRAC_1_SQRT_2);
        // TODO: understand how to do this assertion
        //assert_f32_near!(result.data[2], 0.0, 100000);
    }

    #[test]
    fn test_get_temperature_and_phi_arrays(){
        let s = Simulation::new(100, 100);

        let temperature_rgb = s.get_temperature_rgb();
        assert_eq!(temperature_rgb.len(), 100 * 100 * 4);

        assert_eq!(temperature_rgb[0], COLORMAP_COOL_TO_WARM[0].r);
        assert_eq!(temperature_rgb[1], COLORMAP_COOL_TO_WARM[0].g);
        assert_eq!(temperature_rgb[2], COLORMAP_COOL_TO_WARM[0].b);
        assert_eq!(temperature_rgb[3], COLORMAP_COOL_TO_WARM[0].a);

        let phi_rgb = s.get_phi_rgb();
        assert_eq!(phi_rgb.len(), 100 * 100 * 4);
        assert_eq!(phi_rgb[0], COLORMAP_BLUE_TO_WHITE[0].r);
        assert_eq!(phi_rgb[1], COLORMAP_BLUE_TO_WHITE[0].g);
        assert_eq!(phi_rgb[2], COLORMAP_BLUE_TO_WHITE[0].b);
        assert_eq!(phi_rgb[3], COLORMAP_BLUE_TO_WHITE[0].a);

        assert_eq!(phi_rgb[50 * 4 * 100 + 4 * 50], COLORMAP_BLUE_TO_WHITE[9].r);
        assert_eq!(phi_rgb[50 * 4 * 100 + 4 * 50 + 1], COLORMAP_BLUE_TO_WHITE[9].g);
        assert_eq!(phi_rgb[50 * 4 * 100 + 4 * 50 + 2], COLORMAP_BLUE_TO_WHITE[9].b);
        assert_eq!(phi_rgb[50 * 4 * 100 + 4 * 50 + 3], COLORMAP_BLUE_TO_WHITE[9].a);
    }

}
