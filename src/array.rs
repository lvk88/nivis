use itertools::Itertools;

// This is just a 2D array
pub struct Array2D{
    pub size: [usize; 2],
    pub data: Vec<f32>
}

impl Array2D{
    pub fn ravel(&self, i: usize, j: usize) -> usize{
        j * self.size[0] + i
    }

    pub fn value(&self, i: usize, j: usize) -> f32{
        self.data[self.ravel(i,j)]
    }
}

pub fn atan2(y: &Array2D, x: &Array2D) -> Array2D{
    let data = y.data.iter().zip(x.data.iter()).map(|(y, x)|{
        y.atan2(*x)
    }).collect();

    Array2D{
        data,
        ..*y
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

}
