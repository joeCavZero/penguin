use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PengHeapPtr(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PengNamePoolPtr(pub usize);

impl From<usize> for PengHeapPtr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PengHeapPtr> for usize {
    fn from(value: PengHeapPtr) -> Self {
        value.0
    }
}

impl Add<usize> for PengHeapPtr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for PengHeapPtr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for PengHeapPtr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for PengHeapPtr {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl Mul<usize> for PengHeapPtr {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self {
        Self(self.0 * rhs)
    }
}

impl MulAssign<usize> for PengHeapPtr {
    fn mul_assign(&mut self, rhs: usize) {
        self.0 *= rhs;
    }
}

impl Div<usize> for PengHeapPtr {
    type Output = Self;

    fn div(self, rhs: usize) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<usize> for PengHeapPtr {
    fn div_assign(&mut self, rhs: usize) {
        self.0 /= rhs;
    }
}

impl Rem<usize> for PengHeapPtr {
    type Output = Self;

    fn rem(self, rhs: usize) -> Self {
        Self(self.0 % rhs)
    }
}

impl RemAssign<usize> for PengHeapPtr {
    fn rem_assign(&mut self, rhs: usize) {
        self.0 %= rhs;
    }
}

impl From<usize> for PengNamePoolPtr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PengNamePoolPtr> for usize {
    fn from(value: PengNamePoolPtr) -> Self {
        value.0
    }
}

impl Add<usize> for PengNamePoolPtr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for PengNamePoolPtr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for PengNamePoolPtr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for PengNamePoolPtr {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl Mul<usize> for PengNamePoolPtr {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self {
        Self(self.0 * rhs)
    }
}

impl MulAssign<usize> for PengNamePoolPtr {
    fn mul_assign(&mut self, rhs: usize) {
        self.0 *= rhs;
    }
}

impl Div<usize> for PengNamePoolPtr {
    type Output = Self;

    fn div(self, rhs: usize) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<usize> for PengNamePoolPtr {
    fn div_assign(&mut self, rhs: usize) {
        self.0 /= rhs;
    }
}

impl Rem<usize> for PengNamePoolPtr {
    type Output = Self;

    fn rem(self, rhs: usize) -> Self {
        Self(self.0 % rhs)
    }
}

impl RemAssign<usize> for PengNamePoolPtr {
    fn rem_assign(&mut self, rhs: usize) {
        self.0 %= rhs;
    }
}