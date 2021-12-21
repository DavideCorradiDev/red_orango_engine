use super::{convert, HomogeneousMatrix2, HomogeneousMatrix3, RealField, Rotation2, Vector2};

pub fn translation2<N: RealField + Copy>(translation: &Vector2<N>) -> HomogeneousMatrix2<N> {
    let mut out = HomogeneousMatrix2::<N>::identity();
    out[(0, 2)] = translation[0];
    out[(1, 2)] = translation[1];
    out
}

pub fn rotation2<N: RealField + Copy>(rotation: &Rotation2<N>) -> HomogeneousMatrix2<N> {
    rotation.to_homogeneous()
}

pub fn scale2<N: RealField + Copy>(scale: &Vector2<N>) -> HomogeneousMatrix2<N> {
    let mut out = HomogeneousMatrix2::<N>::identity();
    out[(0, 0)] = scale[0];
    out[(1, 1)] = scale[1];
    out
}

pub fn shear2<N: RealField + Copy>(shear: &Vector2<N>) -> HomogeneousMatrix2<N> {
    let mut out = HomogeneousMatrix2::<N>::identity();
    out[(0, 1)] = shear[0];
    out[(1, 0)] = shear[1];
    out
}

pub fn ortographic_projection2<N: RealField + Copy>(
    left: N,
    right: N,
    bottom: N,
    top: N,
) -> HomogeneousMatrix2<N> {
    let mut out = HomogeneousMatrix2::<N>::identity();
    out[(0, 0)] = convert::<_, N>(2.0) / (right - left);
    out[(0, 2)] = -(right + left) / (right - left);
    out[(1, 1)] = convert::<_, N>(2.0) / (top - bottom);
    out[(1, 2)] = -(top + bottom) / (top - bottom);
    out
}

pub fn transform2_to_transform3<N: RealField + Copy>(
    transform2: &HomogeneousMatrix2<N>,
) -> HomogeneousMatrix3<N> {
    let mut out = HomogeneousMatrix3::<N>::identity();
    out[(0, 0)] = transform2[(0, 0)];
    out[(0, 1)] = transform2[(0, 1)];
    out[(0, 3)] = transform2[(0, 2)];
    out[(1, 0)] = transform2[(1, 0)];
    out[(1, 1)] = transform2[(1, 1)];
    out[(1, 3)] = transform2[(1, 2)];
    out[(3, 0)] = transform2[(2, 0)];
    out[(3, 1)] = transform2[(2, 1)];
    out[(3, 3)] = transform2[(2, 2)];
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn test_translation2() {
        let res = translation2(&Vector2::<f32>::new(2., 3.));
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
        let res = scale2(&Vector2::<f32>::new(2., 3.));
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
        let res = shear2(&Vector2::<f32>::new(2., 3.));
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

    #[test]
    fn test_transform2_to_transform3() {
        let transform2 = HomogeneousMatrix2::new(1., 2., 3., 4., 5., 6., 7., 8., 9.);
        let res = transform2_to_transform3::<f32>(&transform2);
        expect_that!(&res[(0, 0)], close_to(1., 1e-6));
        expect_that!(&res[(0, 1)], close_to(2., 1e-6));
        expect_that!(&res[(0, 2)], close_to(0., 1e-6));
        expect_that!(&res[(0, 3)], close_to(3., 1e-6));
        expect_that!(&res[(1, 0)], close_to(4., 1e-6));
        expect_that!(&res[(1, 1)], close_to(5., 1e-6));
        expect_that!(&res[(1, 2)], close_to(0., 1e-6));
        expect_that!(&res[(1, 3)], close_to(6., 1e-6));
        expect_that!(&res[(2, 0)], close_to(0., 1e-6));
        expect_that!(&res[(2, 1)], close_to(0., 1e-6));
        expect_that!(&res[(2, 2)], close_to(1., 1e-6));
        expect_that!(&res[(2, 3)], close_to(0., 1e-6));
        expect_that!(&res[(3, 0)], close_to(7., 1e-6));
        expect_that!(&res[(3, 1)], close_to(8., 1e-6));
        expect_that!(&res[(3, 2)], close_to(0., 1e-6));
        expect_that!(&res[(3, 3)], close_to(9., 1e-6));
    }
}
