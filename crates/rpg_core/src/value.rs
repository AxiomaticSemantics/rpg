use serde_derive::{Deserialize as De, Serialize as Ser};
use std::{convert::From, ops};

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ser, De)]
pub enum ValueKind {
    U32,
    U64,
    F32,
    F64,
}

#[derive(Debug, Copy, Clone, PartialOrd, Ser, De)]
pub enum Value {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl PartialEq for Value {
    fn eq(&self, rhs: &Self) -> bool {
        match self {
            Self::U32(v) => *v == *rhs.u32(),
            Self::U64(v) => *v == *rhs.u64(),
            Self::F32(v) => (f32::max(*v, *rhs.f32()) - f32::min(*v, *rhs.f32())) <= f32::EPSILON,
            Self::F64(v) => (f64::max(*v, *rhs.f64()) - f64::min(*v, *rhs.f64())) <= f64::EPSILON,
        }
    }
}

impl Eq for Value {}

impl Value {
    pub fn zero(kind: ValueKind) -> Self {
        match kind {
            ValueKind::U32 => Self::U32(0),
            ValueKind::U64 => Self::U64(0),
            ValueKind::F32 => Self::F32(0.),
            ValueKind::F64 => Self::F64(0.),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::U32(v) => v.to_string(),
            Self::U64(v) => v.to_string(),
            Self::F32(v) => v.to_string(),
            Self::F64(v) => v.to_string(),
        }
    }

    pub fn u32(&self) -> &u32 {
        if let Self::U32(ref value) = self {
            value
        } else {
            panic!("Bad variant expected u32");
        }
    }

    pub fn u32_mut(&mut self) -> &mut u32 {
        if let Self::U32(ref mut value) = self {
            value
        } else {
            panic!("Bad variant expected u32");
        }
    }

    pub fn u64(&self) -> &u64 {
        if let Self::U64(ref value) = self {
            value
        } else {
            panic!("Bad variant expected u64");
        }
    }

    pub fn u64_mut(&mut self) -> &mut u64 {
        if let Self::U64(ref mut value) = self {
            value
        } else {
            panic!("Bad variant expected u64");
        }
    }

    pub fn f32(&self) -> &f32 {
        if let Self::F32(ref value) = self {
            value
        } else {
            panic!("Bad variant expected f32");
        }
    }

    pub fn f32_mut(&mut self) -> &mut f32 {
        if let Self::F32(ref mut value) = self {
            value
        } else {
            panic!("Bad variant expected f32");
        }
    }

    pub fn f64(&self) -> &f64 {
        if let Self::F64(ref value) = self {
            value
        } else {
            panic!("Bad variant expected f64");
        }
    }

    pub fn f64_mut(&mut self) -> &mut f64 {
        if let Self::F64(ref mut value) = self {
            value
        } else {
            panic!("Bad variant expected f64");
        }
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl PartialOrd<u32> for Value {
    fn partial_cmp(&self, rhs: &u32) -> Option<std::cmp::Ordering> {
        Some(self.u32().cmp(rhs))
    }
}

impl PartialOrd<u64> for Value {
    fn partial_cmp(&self, rhs: &u64) -> Option<std::cmp::Ordering> {
        Some(self.u64().cmp(rhs))
    }
}

impl PartialOrd<f32> for Value {
    fn partial_cmp(&self, rhs: &f32) -> Option<std::cmp::Ordering> {
        Some(self.f32().total_cmp(rhs))
    }
}

impl PartialOrd<f64> for Value {
    fn partial_cmp(&self, rhs: &f64) -> Option<std::cmp::Ordering> {
        Some(self.f64().total_cmp(rhs))
    }
}

impl PartialEq<u32> for Value {
    fn eq(&self, rhs: &u32) -> bool {
        *self.u32() == *rhs
    }
}

impl PartialEq<u64> for Value {
    fn eq(&self, rhs: &u64) -> bool {
        *self.u64() == *rhs
    }
}

impl PartialEq<f32> for Value {
    fn eq(&self, rhs: &f32) -> bool {
        *self.f32() == *rhs
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, rhs: &f64) -> bool {
        *self.f64() == *rhs
    }
}

impl ops::Add<Self> for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match self {
            Self::U32(value) => Self::U32(value + *rhs.u32()),
            Self::U64(value) => Self::U64(value + *rhs.u64()),
            Self::F32(value) => Self::F32(value + *rhs.f32()),
            Self::F64(value) => Self::F64(value + *rhs.f64()),
        }
    }
}

impl ops::Sub<Self> for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        match self {
            Self::U32(value) => Self::U32(value - *rhs.u32()),
            Self::U64(value) => Self::U64(value - *rhs.u64()),
            Self::F32(value) => Self::F32(value - *rhs.f32()),
            Self::F64(value) => Self::F64(value - *rhs.f64()),
        }
    }
}

impl ops::Mul<Self> for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        match self {
            Self::U32(value) => Self::U32(value * *rhs.u32()),
            Self::U64(value) => Self::U64(value * *rhs.u64()),
            Self::F32(value) => Self::F32(value * *rhs.f32()),
            Self::F64(value) => Self::F64(value * *rhs.f64()),
        }
    }
}

impl ops::Div<Self> for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        match self {
            Self::U32(value) => Self::U32(value / *rhs.u32()),
            Self::U64(value) => Self::U64(value / *rhs.u64()),
            Self::F32(value) => Self::F32(value / *rhs.f32()),
            Self::F64(value) => Self::F64(value / *rhs.f64()),
        }
    }
}

impl ops::Add<u32> for Value {
    type Output = Self;

    fn add(self, rhs: u32) -> Self {
        match self {
            Self::U32(value) => Self::U32(value + rhs),
            _ => panic!("Bad variant in add expected u32"),
        }
    }
}

impl ops::Add<u64> for Value {
    type Output = Self;

    fn add(self, rhs: u64) -> Self {
        match self {
            Self::U64(value) => Self::U64(value + rhs),
            _ => panic!("Bad variant in add expected u64"),
        }
    }
}

impl ops::Add<f32> for Value {
    type Output = Self;

    fn add(self, rhs: f32) -> Self {
        match self {
            Self::F32(value) => Self::F32(value + rhs),
            _ => panic!("Bad variant in add expected f32"),
        }
    }
}

impl ops::Add<f64> for Value {
    type Output = Self;

    fn add(self, rhs: f64) -> Self {
        match self {
            Self::F64(value) => Self::F64(value + rhs),
            _ => panic!("Bad variant in add expected f64"),
        }
    }
}

impl ops::Sub<u32> for Value {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self {
        match self {
            Self::U32(value) => Self::U32(value - rhs),
            _ => panic!("Bad variant in sub expected u32"),
        }
    }
}

impl ops::Sub<u64> for Value {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self {
        match self {
            Self::U64(value) => Self::U64(value - rhs),
            _ => panic!("Bad variant in sub expected u64"),
        }
    }
}

impl ops::Sub<f32> for Value {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self {
        match self {
            Self::F32(value) => Self::F32(value - rhs),
            _ => panic!("Bad variant in sub expected f32"),
        }
    }
}

impl ops::Sub<f64> for Value {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self {
        match self {
            Self::F64(value) => Self::F64(value - rhs),
            _ => panic!("Bad variant in sub expected f64"),
        }
    }
}

impl ops::Mul<u32> for Value {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self {
        match self {
            Self::U32(value) => Self::U32(value * rhs),
            _ => panic!("Bad variant in mul expected u32"),
        }
    }
}

impl ops::Mul<u64> for Value {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self {
        match self {
            Self::U64(value) => Self::U64(value * rhs),
            _ => panic!("Bad variant in mul expected u64"),
        }
    }
}

impl ops::Mul<f32> for Value {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        match self {
            Self::F32(value) => Self::F32(value * rhs),
            _ => panic!("Bad variant in mul expected f32"),
        }
    }
}

impl ops::Mul<f64> for Value {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        match self {
            Self::F64(value) => Self::F64(value * rhs),
            _ => panic!("Bad variant in mul expected f64"),
        }
    }
}

impl ops::Div<u32> for Value {
    type Output = Self;

    fn div(self, rhs: u32) -> Self {
        match self {
            Self::U32(value) => Self::U32(value / rhs),
            _ => panic!("Bad variant in div expected u32"),
        }
    }
}

impl ops::Div<u64> for Value {
    type Output = Self;

    fn div(self, rhs: u64) -> Self {
        match self {
            Self::U64(value) => Self::U64(value / rhs),
            _ => panic!("Bad variant in div epxected u64"),
        }
    }
}

impl ops::Div<f32> for Value {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        match self {
            Self::F32(value) => Self::F32(value / rhs),
            _ => panic!("Bad variant in div expected f32"),
        }
    }
}

impl ops::Div<f64> for Value {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        match self {
            Self::F64(value) => Self::F64(value / rhs),
            _ => panic!("Bad variant in div expected f64"),
        }
    }
}

impl ops::AddAssign<Self> for Value {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl ops::AddAssign<u32> for Value {
    fn add_assign(&mut self, rhs: u32) {
        *self = *self + rhs;
    }
}

impl ops::AddAssign<u64> for Value {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl ops::AddAssign<f32> for Value {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}

impl ops::AddAssign<f64> for Value {
    fn add_assign(&mut self, rhs: f64) {
        *self = *self + rhs;
    }
}

impl ops::SubAssign<Self> for Value {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl ops::SubAssign<u32> for Value {
    fn sub_assign(&mut self, rhs: u32) {
        *self = *self - rhs;
    }
}

impl ops::SubAssign<u64> for Value {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl ops::SubAssign<f32> for Value {
    fn sub_assign(&mut self, rhs: f32) {
        *self = *self - rhs;
    }
}

impl ops::SubAssign<f64> for Value {
    fn sub_assign(&mut self, rhs: f64) {
        *self = *self - rhs;
    }
}

impl ops::MulAssign<Self> for Value {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl ops::MulAssign<u32> for Value {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

impl ops::MulAssign<u64> for Value {
    fn mul_assign(&mut self, rhs: u64) {
        *self = *self * rhs;
    }
}

impl ops::MulAssign<f32> for Value {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl ops::MulAssign<f64> for Value {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl ops::DivAssign<Self> for Value {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl ops::DivAssign<u32> for Value {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

impl ops::DivAssign<u64> for Value {
    fn div_assign(&mut self, rhs: u64) {
        *self = *self / rhs;
    }
}

impl ops::DivAssign<f32> for Value {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl ops::DivAssign<f64> for Value {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}
