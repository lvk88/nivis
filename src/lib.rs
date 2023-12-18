use wasm_bindgen::prelude::*;

use itertools::Itertools;

// This is just a 2D array
pub struct Array2D{
    size: [usize; 2],
    data: Vec<f32>
}

#[wasm_bindgen]
pub struct Simulation{
    pub width: usize,
    pub height: usize,
    dx: f32,
    dy: f32,
    pub kappa: f32,
    pub delta: f32,
    temperature: Array2D,
    phi: Array2D
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
        let kappa = 1.6;
        let delta = 0.04;

        // The size of the temperature and phi fields is one node more on each sides
        // to accomodate for ghost nodes
        let temperature = new_array_with_function(width + 2, height + 2, dx, dy, |_,_| 0.0);
        let phi = new_array_with_function(width + 2, height + 2, dx, dy, |_,_| 0.0);

        Simulation{
            width,
            height,
            dx,
            dy,
            kappa,
            delta,
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

        let dphidx = diff_x(&self.phi, self.dx);
        let dphidy = diff_y(&self.phi, self.dy);
        let laplace_phi = laplace(&self.phi, self.dx, self.dy);

        let laplace_temperature = laplace(&self.temperature, self.dx, self.dy);

        let theta = atan2(&dphidy, &dphidx);

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

        let term1: Vec<f32> = epsilon_x_depsilondtheta.iter().zip(dphidx.data.iter()).map(|(lhs, rhs)|{
            lhs * rhs
        }).collect();

        let term1 = Array2D{
            data: term1,
            ..self.temperature
        };

        let term1 = diff_y(&term1, self.dy);

        let term2: Vec<f32> = epsilon_x_depsilondtheta.iter().zip(dphidy.data.iter()).map(|(lhs, rhs)|{
            lhs * rhs
        }).collect();

        let term2 = Array2D{
            data: term2,
            ..self.temperature
        };

        let term2 = diff_x(&term2, self.dx);

        let m: Vec<f32> = self.temperature.data.iter().map(|val|{
            alpha / std::f32::consts::PI * (gamma * (teq - val)).atan()
        }).collect();

        let new_phi: Vec<f32> = (0..self.phi.data.len()).map(|i|{
            self.phi.data[i] + (delta_t / tau) * (term1.data[i] - term2.data[i] + epsilon[i] * epsilon[i] * laplace_phi.data[i] + self.phi.data[i] * (1.0 - self.phi.data[i]) * (self.phi.data[i] - 0.5 + m[i]))
        }).collect();

        let new_temperature: Vec<f32> = (0..self.temperature.data.len()).map(|i|{
            self.temperature.data[i] + delta_t * laplace_temperature.data[i] + self.kappa * (new_phi[i] - self.phi.data[i])
        }).collect();

        self.temperature.data = new_temperature;
        self.phi.data = new_phi;
    }

    pub fn reset(&mut self){
        let temperature = new_array_with_function(self.width + 2, self.height + 2, self.dx, self.dy, |_,_|{0.0});

        let phi = new_array_with_function(self.width + 2, self.height + 2, self.dx, self.dy, |_,_|{0.0});

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
                let index = self.phi.ravel(cx + 1, cy + 1);
                self.phi.data[index] = 1.0;
            }
        }
    }
}

pub fn new_array_with_function(nx: usize, ny: usize, dx: f32, dy: f32, func: impl Fn(f32, f32) -> f32) -> Array2D{
    let data = (0..ny).cartesian_product(0..nx).map(|(j, i)|{
        let x = i as f32 * dx;
        let y = j as f32 * dy;
        func(x,y)
    }).collect();

    Array2D{
        size: [nx, ny],
        data
    }

}

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

fn atan2(y: &Array2D, x: &Array2D) -> Array2D{
    let data = y.data.iter().zip(x.data.iter()).map(|(y, x)|{
        y.atan2(*x)
    }).collect();

    Array2D{
        data,
        ..*y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_float_eq::*;

    #[test]
    fn create_array2d_with_function() {
        let grid_data = new_array_with_function(3, 4, 0.1, 0.2, |x,y|{ x + y });

        assert_f32_near!(grid_data.value(0,0), 0.0);
        assert_f32_near!(grid_data.value(1,0), 0.1);
        assert_f32_near!(grid_data.value(0,1), 0.2);
        assert_f32_near!(grid_data.value(1,1), 0.3);
        assert_f32_near!(grid_data.value(2,3), 0.8);
    }

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
    fn test_get_temperature_and_phi_arrays(){
        let mut s = Simulation::new(100, 100);
        s.add_seed(50, 50);

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
