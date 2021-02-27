use crate::{geometry2, geometry3, matrix::RealField};

pub trait ToHomogeneousVector3<N: RealField> {
    fn to_homogeneous3(&self) -> geometry3::HomogeneousVector<N>;
}

impl<N: RealField> ToHomogeneousVector3<N> for geometry2::HomogeneousVector<N> {
    fn to_homogeneous3(&self) -> geometry3::HomogeneousVector<N> {
        geometry3::HomogeneousVector::<N>::new(self[0], self[1], N::zero(), self[2])
    }
}

pub trait ToHomogeneousMatrix3<N: RealField> {
    fn to_homogeneous3(&self) -> geometry3::HomogeneousMatrix<N>;
}

impl<N: RealField> ToHomogeneousMatrix3<N> for geometry2::HomogeneousMatrix<N> {
    fn to_homogeneous3(&self) -> geometry3::HomogeneousMatrix<N> {
        let mut out = geometry3::HomogeneousMatrix::<N>::identity();
        out[(0, 0)] = self[(0, 0)];
        out[(0, 1)] = self[(0, 1)];
        out[(0, 3)] = self[(0, 2)];
        out[(1, 0)] = self[(1, 0)];
        out[(1, 1)] = self[(1, 1)];
        out[(1, 3)] = self[(1, 2)];
        out
    }
}

macro_rules! implement_to_homogeneous3 {
    ($StructType:ty) => {
        impl<N: RealField> ToHomogeneousMatrix3<N> for $StructType {
            fn to_homogeneous3(&self) -> geometry3::HomogeneousMatrix<N> {
                self.to_homogeneous().to_homogeneous3()
            }
        }
    };
}

implement_to_homogeneous3!(geometry2::Affine<N>);
implement_to_homogeneous3!(geometry2::Isometry<N>);
implement_to_homogeneous3!(geometry2::Projective<N>);
implement_to_homogeneous3!(geometry2::Rotation<N>);
implement_to_homogeneous3!(geometry2::Similarity<N>);
implement_to_homogeneous3!(geometry2::Transform<N>);
implement_to_homogeneous3!(geometry2::Translation<N>);
implement_to_homogeneous3!(geometry2::OrthographicProjection<N>);

#[cfg(test)]
mod tests {
    use super::*;

    use galvanic_assert::{matchers::*, *};

    #[test]
    fn homogeneous_vector_2_to_homogeneous_vector_3() {
        let v2 = geometry2::HomogeneousVector::new(10., 11., 12.);
        let v3 = v2.to_homogeneous3();

        expect_that!(&v3[0], close_to(v2[0], 1e-12));
        expect_that!(&v3[1], close_to(v2[1], 1e-12));
        expect_that!(&v3[2], close_to(0., 1e-12));
        expect_that!(&v3[3], close_to(v2[2], 1e-12));
    }

    #[test]
    fn homogeneous_matrix_2_to_homogeneous_matrix_3() {
        let m2 = geometry2::HomogeneousMatrix::new(10., 11., 12., 13., 14., 15., 16., 17., 18.);
        let m3 = m2.to_homogeneous3();

        expect_that!(&m3[(0, 0)], close_to(m2[(0, 0)], 1e-12));
        expect_that!(&m3[(0, 1)], close_to(m2[(0, 1)], 1e-12));
        expect_that!(&m3[(0, 2)], close_to(0., 1e-12));
        expect_that!(&m3[(0, 3)], close_to(m2[(0, 2)], 1e-12));
        expect_that!(&m3[(1, 0)], close_to(m2[(1, 0)], 1e-12));
        expect_that!(&m3[(1, 1)], close_to(m2[(1, 1)], 1e-12));
        expect_that!(&m3[(1, 2)], close_to(0., 1e-12));
        expect_that!(&m3[(1, 3)], close_to(m2[(1, 2)], 1e-12));
        expect_that!(&m3[(2, 0)], close_to(0., 1e-12));
        expect_that!(&m3[(2, 1)], close_to(0., 1e-12));
        expect_that!(&m3[(2, 2)], close_to(1., 1e-12));
        expect_that!(&m3[(2, 3)], close_to(0., 1e-12));
        expect_that!(&m3[(3, 0)], close_to(0., 1e-12));
        expect_that!(&m3[(3, 1)], close_to(0., 1e-12));
        expect_that!(&m3[(3, 2)], close_to(0., 1e-12));
        expect_that!(&m3[(3, 3)], close_to(1., 1e-12));
    }
}
