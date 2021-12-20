use super::{convert, HomogeneousMatrix2, RealField, Rotation2, Scale2, Shear2, Translation2};

pub fn translation2<N: RealField + Copy>(translation: &Translation2<N>) -> HomogeneousMatrix2<N> {
    let mut matrix = HomogeneousMatrix2::<N>::identity();
    matrix[(0, 2)] = translation[0];
    matrix[(1, 2)] = translation[1];
    matrix
}

pub fn rotation2<N: RealField + Copy>(rotation: &Rotation2<N>) -> HomogeneousMatrix2<N> {
    rotation.to_homogeneous()
}

pub fn scale2<N: RealField + Copy>(scale: &Scale2<N>) -> HomogeneousMatrix2<N> {
    let mut matrix = HomogeneousMatrix2::<N>::identity();
    matrix[(0, 0)] = scale[0];
    matrix[(1, 1)] = scale[1];
    matrix
}

pub fn shear2<N: RealField + Copy>(shear: &Shear2<N>) -> HomogeneousMatrix2<N> {
    let mut matrix = HomogeneousMatrix2::<N>::identity();
    matrix[(0, 1)] = shear[0];
    matrix[(1, 0)] = shear[1];
    matrix
}

pub fn ortographic_projection2<N: RealField + Copy>(
    left: N,
    right: N,
    bottom: N,
    top: N,
) -> HomogeneousMatrix2<N> {
    let mut matrix = HomogeneousMatrix2::<N>::identity();
    matrix[(0, 0)] = convert::<_, N>(2.0) / (right - left);
    matrix[(0, 2)] = -(right + left) / (right - left);
    matrix[(1, 1)] = convert::<_, N>(2.0) / (top - bottom);
    matrix[(1, 2)] = -(top + bottom) / (top - bottom);
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn test_translation2() {
        let res = translation2(&Translation2::<f32>::new(2., 3.));
        expect_that!(&res[(0, 0)], close_to(1., 1e-6));
        expect_that!(&res[(0, 1)], close_to(0., 1e-6));
        expect_that!(&res[(0, 2)], close_to(2., 1e-6));
        expect_that!(&res[(1, 0)], close_to(0., 1e-6));
        expect_that!(&res[(1, 1)], close_to(1., 1e-6));
        expect_that!(&res[(1, 2)], close_to(3., 1e-6));
        expect_that!(&res[(2, 0)], close_to(0., 1e-6));
        expect_that!(&res[(2, 1)], close_to(0., 1e-6));
        expect_that!(&res[(2, 2)], close_to(1., 1e-6));
    }

    #[test]
    fn test_rotation2() {
        let res = rotation2(&Rotation2::<f32>::from_angle(1.));
        expect_that!(&res[(0, 0)], close_to(0.540302, 1e-6));
        expect_that!(&res[(0, 1)], close_to(-0.841471, 1e-6));
        expect_that!(&res[(0, 2)], close_to(0., 1e-6));
        expect_that!(&res[(1, 0)], close_to(0.841471, 1e-6));
        expect_that!(&res[(1, 1)], close_to(0.540302, 1e-6));
        expect_that!(&res[(1, 2)], close_to(0., 1e-6));
        expect_that!(&res[(2, 0)], close_to(0., 1e-6));
        expect_that!(&res[(2, 1)], close_to(0., 1e-6));
        expect_that!(&res[(2, 2)], close_to(1., 1e-6));
    }

    #[test]
    fn test_scale2() {
        let res = scale2(&Scale2::<f32>::new(2., 3.));
        expect_that!(&res[(0, 0)], close_to(2., 1e-6));
        expect_that!(&res[(0, 1)], close_to(0., 1e-6));
        expect_that!(&res[(0, 2)], close_to(0., 1e-6));
        expect_that!(&res[(1, 0)], close_to(0., 1e-6));
        expect_that!(&res[(1, 1)], close_to(3., 1e-6));
        expect_that!(&res[(1, 2)], close_to(0., 1e-6));
        expect_that!(&res[(2, 0)], close_to(0., 1e-6));
        expect_that!(&res[(2, 1)], close_to(0., 1e-6));
        expect_that!(&res[(2, 2)], close_to(1., 1e-6));
    }

    #[test]
    fn test_shear2() {
        let res = shear2(&Shear2::<f32>::new(2., 3.));
        expect_that!(&res[(0, 0)], close_to(1., 1e-6));
        expect_that!(&res[(0, 1)], close_to(2., 1e-6));
        expect_that!(&res[(0, 2)], close_to(0., 1e-6));
        expect_that!(&res[(1, 0)], close_to(3., 1e-6));
        expect_that!(&res[(1, 1)], close_to(1., 1e-6));
        expect_that!(&res[(1, 2)], close_to(0., 1e-6));
        expect_that!(&res[(2, 0)], close_to(0., 1e-6));
        expect_that!(&res[(2, 1)], close_to(0., 1e-6));
        expect_that!(&res[(2, 2)], close_to(1., 1e-6));
    }

    #[test]
    fn test_ortographic_projection2() {
        let res = ortographic_projection2::<f32>(1., 5., 2., 11.);
        expect_that!(&res[(0, 0)], close_to(0.5, 1e-6));
        expect_that!(&res[(0, 1)], close_to(0., 1e-6));
        expect_that!(&res[(0, 2)], close_to(-1.5, 1e-6));
        expect_that!(&res[(1, 0)], close_to(0., 1e-6));
        expect_that!(&res[(1, 1)], close_to(0.222222, 1e-6));
        expect_that!(&res[(1, 2)], close_to(-1.444444, 1e-6));
        expect_that!(&res[(2, 0)], close_to(0., 1e-6));
        expect_that!(&res[(2, 1)], close_to(0., 1e-6));
        expect_that!(&res[(2, 2)], close_to(1., 1e-6));
    }
}
