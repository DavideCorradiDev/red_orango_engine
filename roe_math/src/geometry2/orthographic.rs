#[cfg(feature = "serde-serialize")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use nalgebra::base::helper;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::{conversion::convert, matrix::RealField};

use super::{HomogeneousMatrix, Point, Projective, Vector};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OrthographicProjection<N: RealField> {
    matrix: HomogeneousMatrix<N>,
}

impl<N: RealField> OrthographicProjection<N> {
    #[inline]
    pub fn new(left: N, right: N, bottom: N, top: N) -> Self {
        let matrix = HomogeneousMatrix::<N>::identity();
        let mut res = Self::from_matrix_unchecked(matrix);
        res.set_left_and_right(left, right);
        res.set_bottom_and_top(bottom, top);
        res
    }

    #[inline]
    pub fn from_matrix_unchecked(matrix: HomogeneousMatrix<N>) -> Self {
        Self { matrix }
    }

    #[inline]
    pub fn inverse(&self) -> HomogeneousMatrix<N> {
        let mut res = self.to_homogeneous();

        let inv_m11 = N::one() / self.matrix[(0, 0)];
        let inv_m22 = N::one() / self.matrix[(1, 1)];

        res[(0, 0)] = inv_m11;
        res[(1, 1)] = inv_m22;

        res[(0, 2)] = -self.matrix[(0, 2)] * inv_m11;
        res[(1, 2)] = -self.matrix[(1, 2)] * inv_m22;
        res
    }

    #[inline]
    pub fn to_homogeneous(&self) -> HomogeneousMatrix<N> {
        self.matrix
    }

    #[inline]
    pub fn as_matrix(&self) -> &HomogeneousMatrix<N> {
        &self.matrix
    }

    #[inline]
    pub fn as_projective(&self) -> &Projective<N> {
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub fn to_projective(&self) -> Projective<N> {
        Projective::from_matrix_unchecked(self.matrix)
    }

    #[inline]
    pub fn into_inner(self) -> HomogeneousMatrix<N> {
        self.matrix
    }

    #[inline]
    pub fn left(&self) -> N {
        (-N::one() - self.matrix[(0, 2)]) / self.matrix[(0, 0)]
    }

    #[inline]
    pub fn right(&self) -> N {
        (N::one() - self.matrix[(0, 2)]) / self.matrix[(0, 0)]
    }

    #[inline]
    pub fn bottom(&self) -> N {
        (-N::one() - self.matrix[(1, 2)]) / self.matrix[(1, 1)]
    }

    #[inline]
    pub fn top(&self) -> N {
        (N::one() - self.matrix[(1, 2)]) / self.matrix[(1, 1)]
    }

    #[inline]
    pub fn project_point(&self, p: &Point<N>) -> Point<N> {
        Point::new(
            self.matrix[(0, 0)] * p[0] + self.matrix[(0, 2)],
            self.matrix[(1, 1)] * p[1] + self.matrix[(1, 2)],
        )
    }

    #[inline]
    pub fn unproject_point(&self, p: &Point<N>) -> Point<N> {
        Point::new(
            (p[0] - self.matrix[(0, 2)]) / self.matrix[(0, 0)],
            (p[1] - self.matrix[(1, 2)]) / self.matrix[(1, 1)],
        )
    }

    #[inline]
    pub fn project_vector(&self, p: &Vector<N>) -> Vector<N> {
        Vector::new(self.matrix[(0, 0)] * p[0], self.matrix[(1, 1)] * p[1])
    }

    #[inline]
    pub fn set_left(&mut self, left: N) {
        let right = self.right();
        self.set_left_and_right(left, right);
    }

    #[inline]
    pub fn set_right(&mut self, right: N) {
        let left = self.left();
        self.set_left_and_right(left, right);
    }

    #[inline]
    pub fn set_bottom(&mut self, bottom: N) {
        let top = self.top();
        self.set_bottom_and_top(bottom, top);
    }

    #[inline]
    pub fn set_top(&mut self, top: N) {
        let bottom = self.bottom();
        self.set_bottom_and_top(bottom, top);
    }

    #[inline]
    pub fn set_left_and_right(&mut self, left: N, right: N) {
        assert!(
            left != right,
            "The left corner must not be equal to the right corner."
        );
        self.matrix[(0, 0)] = convert::<_, N>(2.0) / (right - left);
        self.matrix[(0, 2)] = -(right + left) / (right - left);
    }

    #[inline]
    pub fn set_bottom_and_top(&mut self, bottom: N, top: N) {
        assert!(
            bottom != top,
            "The top corner must not be equal to the bottom corner."
        );
        self.matrix[(1, 1)] = convert::<_, N>(2.0) / (top - bottom);
        self.matrix[(1, 2)] = -(top + bottom) / (top - bottom);
    }
}

#[cfg(feature = "serde-serialize")]
impl<N: RealField + Serialize> Serialize for OrthographicProjection<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.matrix.serialize(serializer)
    }
}

#[cfg(feature = "serde-serialize")]
impl<'a, N: RealField + Deserialize<'a>> Deserialize<'a> for OrthographicProjection<N> {
    fn deserialize<Des>(deserializer: Des) -> Result<Self, Des::Error>
    where
        Des: Deserializer<'a>,
    {
        let matrix = HomogeneousMatrix::<N>::deserialize(deserializer)?;
        Ok(Self::from_matrix_unchecked(matrix))
    }
}

impl<N: RealField> Distribution<OrthographicProjection<N>> for Standard
where
    Standard: Distribution<N>,
{
    fn sample<R: Rng + ?Sized>(&self, r: &mut R) -> OrthographicProjection<N> {
        let left = r.gen();
        let right = helper::reject_rand(r, |x: &N| *x > left);
        let bottom = r.gen();
        let top = helper::reject_rand(r, |x: &N| *x > bottom);

        OrthographicProjection::new(left, right, bottom, top)
    }
}

impl<N: RealField> From<OrthographicProjection<N>> for HomogeneousMatrix<N> {
    #[inline]
    fn from(orth: OrthographicProjection<N>) -> Self {
        orth.into_inner()
    }
}
