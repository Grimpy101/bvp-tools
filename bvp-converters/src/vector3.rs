use std::{ops::{Sub, Div, Mul, Add}, fmt::Display};

use tinyjson::JsonValue;

#[derive(Clone, Copy)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T
}

impl Vector3<u32> {
    pub fn from_xyz(x: u32, y: u32, z: u32) -> Self {
        return Self { x, y, z };
    }

    pub fn is_any_lt(&self, v: Vector3<u32>) -> bool {
        return self.x < v.x || self.y < v.y || self.z < v.z;
    }

    pub fn is_any_gt(&self, v: Vector3<u32>) -> bool {
        return self.x > v.x || self.y > v.y || self.z > v.z;
    }

    pub fn is_any_div(&self, v: &Vector3<u32>) -> bool {
        return self.x % v.x != 0 || self.y % v.y != 0 || self.z % v.z != 0;
    }

    pub fn min(&self, v: &Vector3<u32>) -> Vector3<u32> {
        return Vector3 {
            x: self.x.min(v.x),
            y: self.y.min(v.y),
            z: self.z.min(v.z)
        };
    }

    pub fn linear_index(i: Vector3<u32>, dim: Vector3<u32>) -> usize {
        let a1 = i.x + i.y * dim.x + i.z * dim.x * dim.y;
        return a1 as usize;
    }

    pub fn to_f64_vec(&self) -> Vec<JsonValue> {
        return vec![(self.x as f64).into(), (self.y as f64).into(), (self.z as f64).into()];
    }

    pub fn multiply_elements(&self) -> u32 {
        return self.x * self.y * self.z;
    }
}

impl Vector3<f32> {
    pub fn ceil(&self) -> Vector3<u32> {
        return Vector3 {
            x: self.x.ceil() as u32,
            y: self.y.ceil() as u32,
            z: self.z.ceil() as u32
        };
    }

    pub fn to_u32(&self) -> Vector3<u32> {
        return Vector3 {
            x: self.x as u32,
            y: self.y as u32,
            z: self.z as u32
        };
    }

    pub fn to_vec(&self) -> Vec<JsonValue> {
        return vec![(self.x as f64).into(), (self.y as f64).into(), (self.z as f64).into()];
    }
}

impl Add for Vector3<u32> {
    type Output = Vector3<u32>;

    fn add(self, rhs: Self) -> Self::Output {
        return Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        };
    }
}

impl Sub for Vector3<u32> {
    type Output = Vector3<u32>;

    fn sub(self, rhs: Self) -> Self::Output {
        return Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        };
    }
}

impl Mul for Vector3<u32> {
    type Output = Vector3<u32>;

    fn mul(self, rhs: Self) -> Self::Output {
        return Vector3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z
        };
    }
}

impl Div for Vector3<u32> {
    type Output = Vector3<f32>;

    fn div(self, rhs: Self) -> Self::Output {
        return Vector3 {
            x: self.x as f32 / rhs.x as f32,
            y: self.y as f32 / rhs.y as f32,
            z: self.z as f32 / rhs.z as f32
        };
    }
}

impl<T: Display> Display for Vector3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} {} {}]", self.x, self.y, self.z)
    }
}