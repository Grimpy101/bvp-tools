use std::{ops::{Sub, Div, Mul, Add}, fmt::{Display, Debug}};

use num_traits::{PrimInt};
use tinyjson::JsonValue;

use crate::errors::JsonError;

#[derive(Clone, Copy)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T
}

impl<T> Vector3<T> {
    pub fn from_xyz(x: T, y: T, z: T) -> Self {
        return Self { x, y, z };
    }
}

impl<T: PartialOrd> Vector3<T> {
    /// Returns true if any component in self is lower than
    /// corresponding component in vector v, else returns false.
    /// * `v` - vector to check againts
    pub fn is_any_lt(&self, v: Vector3<T>) -> bool {
        return self.x < v.x || self.y < v.y || self.z < v.z;
    }

    /// Returns true if any component in self is greater than
    /// corresponding component in vector v, else returns false.
    /// * `v` - vector to check against
    pub fn is_any_gt(&self, v: Vector3<T>) -> bool {
        return self.x > v.x || self.y > v.y || self.z > v.z;
    }
}

impl<T: PrimInt> Vector3<T> {
    /// Returns true if any component in self is not
    /// divisible by the corresponding component in vector v.
    /// * `v` - vector to check against
    pub fn is_any_div(&self, v: &Vector3<T>) -> bool {
        return !(self.x % v.x).is_zero() || !(self.y % v.y).is_zero() || !(self.z % v.z).is_zero();
    }

    /// Returns a new vector containing the minimum
    /// of the corresponding components in both vectors.
    /// * `v` - the other vector
    pub fn min(&self, v: &Vector3<T>) -> Vector3<T> {
        return Vector3 {
            x: self.x.min(v.x),
            y: self.y.min(v.y),
            z: self.z.min(v.z)
        };
    }
}

impl Vector3<u32> {
    /// Based on 3D index vector and dimensions,
    /// calculates the 1D index and returns it.
    /// * `i` - 3D index vector
    /// * `dim` - dimensions of the 3D structure
    pub fn linear_index(i: Vector3<u32>, dim: Vector3<u32>) -> usize {
        let a1 = i.x + i.y * dim.x + i.z * dim.x * dim.y;
        return a1 as usize;
    }

    /// Returns the product of all vector components (u32).
    pub fn multiply_elements(&self) -> u32 {
        return self.x * self.y * self.z;
    }

    /// Creates 3D vector (u32) from JSON array.
    /// * `j` - JSON array of numbers
    pub fn from_json(j: &JsonValue) -> Result<Self, JsonError> {
        match j {
            JsonValue::Array(a) => {
                if a.len() != 3 {
                    return Err(JsonError::NotAVector3(j.clone()));
                }
                let x = match a[0] {
                    JsonValue::Number(n) => {
                        n as u32
                    },
                    _ => {
                        return Err(JsonError::NotANumber(a[0].clone()));
                    }
                };
                let y = match a[1] {
                    JsonValue::Number(n) => {
                        n as u32
                    },
                    _ => {
                        return Err(JsonError::NotANumber(a[1].clone()));
                    }
                };
                let z = match a[2] {
                    JsonValue::Number(n) => {
                        n as u32
                    },
                    _ => {
                        return Err(JsonError::NotANumber(a[2].clone()));
                    }
                };

                let vec = Vector3::from_xyz(x, y, z);

                return Ok(vec);
            },
            _ => ()
        }
        return Err(JsonError::NotAVector3(j.clone()));
    }

    /// Converts 3D vector (u32) to JSON array.
    pub fn to_json(&self) -> JsonValue {
        let v = vec![
            (self.x as f64).into(),
            (self.y as f64).into(),
            (self.z as f64).into()
        ];
        return v.into();
    }
}

impl Vector3<f32> {
    /// Rounds all components of the vector (f32) up.
    /// The returned vector is (u32), so all components
    /// are expected to be positive.
    pub fn ceil(&self) -> Vector3<u32> {
        return Vector3 {
            x: self.x.ceil() as u32,
            y: self.y.ceil() as u32,
            z: self.z.ceil() as u32
        };
    }

    /// Rounds all components of the vector (f32) down.
    /// The returned vector is (u32), so all components
    /// are expected to be positive.
    pub fn to_u32(&self) -> Vector3<u32> {
        return Vector3 {
            x: self.x as u32,
            y: self.y as u32,
            z: self.z as u32
        };
    }

    /// Converts 3D vector (f32) to JSON array.
    pub fn to_json(&self) -> JsonValue {
        let v = vec![
            (self.x as f64).into(),
            (self.y as f64).into(),
            (self.z as f64).into()
        ];
        return v.into();
    }

    /// Creates 3D vector (f32) from JSON array.
    /// * `j` - JSON array of numbers
    pub fn from_json(j: &JsonValue) -> Result<Self, JsonError> {
        match j {
            JsonValue::Array(a) => {
                if a.len() != 3 {
                    return Err(JsonError::NotAVector3(j.clone()));
                }
                let x = match a[0] {
                    JsonValue::Number(n) => {
                        n as f32
                    },
                    _ => {
                        return Err(JsonError::NotANumber(a[0].clone()));
                    }
                };
                let y = match a[1] {
                    JsonValue::Number(n) => {
                        n as f32
                    },
                    _ => {
                        return Err(JsonError::NotANumber(a[1].clone()));
                    }
                };
                let z = match a[2] {
                    JsonValue::Number(n) => {
                        n as f32
                    },
                    _ => {
                        return Err(JsonError::NotANumber(a[2].clone()));
                    }
                };

                let vec = Vector3 { x, y, z };

                return Ok(vec);
            },
            _ => ()
        }
        return Err(JsonError::NotAVector3(j.clone()));
    }
}

impl Into<Vector3<f32>> for Vector3<u32> {
    fn into(self) -> Vector3<f32> {
        return Vector3 {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32
        };
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
        return write!(f, "[{} {} {}]", self.x, self.y, self.z);
    }
}

impl<T: Debug> Debug for Vector3<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "[{:?} {:?} {:?}]", self.x, self.y, self.z);
    }
}