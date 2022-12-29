use std::ops::{Index, Mul, Add, Sub};


#[derive(Copy, Clone)]
pub struct Mat3 (pub [f64; 9]);

impl Mat3 {
    pub fn identity() -> Self {
        Mat3([
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        ])
    }

    pub fn from_slice(vec: &[f64]) -> Self {
        let mut result = [0.0; 9];
        result.copy_from_slice(&vec[..9]);
        Mat3(result)
    }
    
    pub fn transpose(self) -> Self {
        let mut result = [0.0; 9];

        for i in 0..3 {
            for j in 0..3 {
                result[3*i + j] = self[(j, i)];
            }
        }

        Mat3(result)
    }
}

impl Index<(usize, usize)> for Mat3 {
    type Output = f64;

    fn index(&self, (i, j): (usize, usize)) -> &Self::Output {
        &self.0[3*i + j]
    }
}

impl Mul<Mat3> for Mat3 {
    type Output = Mat3;

    fn mul(self, rhs: Mat3) -> Self::Output {
        let mut result = [0.0; 9];

        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[3*i + j] += self[(i, k)]*rhs[(k, j)];
                }
            }
        }

        Mat3(result)
    }
}


#[derive(Copy, Clone)]
pub struct Vec3 (pub [f64; 3]);

impl Vec3 {
    pub fn zero() -> Self {
        Vec3([ 0.0, 0.0, 0.0 ])
    }

    pub fn from_slice(vec: &[f64]) -> Self {
        let mut result = [0.0; 3];
        result.copy_from_slice(&vec[..3]);
        Vec3(result)
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3([
            self[0] + rhs[0],
            self[1] + rhs[1],
            self[2] + rhs[2],
        ])
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3([
            self[0] - rhs[0],
            self[1] - rhs[1],
            self[2] - rhs[2],
        ])
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3([
            self[(0, 0)]*rhs[0] + self[(0, 1)]*rhs[1] + self[(0, 2)]*rhs[2],
            self[(1, 0)]*rhs[0] + self[(1, 1)]*rhs[1] + self[(1, 2)]*rhs[2],
            self[(2, 0)]*rhs[0] + self[(2, 1)]*rhs[1] + self[(2, 2)]*rhs[2],
        ])
    }
}