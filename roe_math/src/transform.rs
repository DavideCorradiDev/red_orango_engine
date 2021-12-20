use super::{HomogeneousMatrix2, RealField, Rotation2, Translation2, Scaling2};

pub fn translation2<N: RealField + Copy>(translation: &Translation2<N>) -> HomogeneousMatrix2<N> {
    let mut matrix = HomogeneousMatrix2::<N>::identity();
    matrix[(0,2)] = translation[0];
    matrix[(1,2)] = translation[1];
    matrix
}

pub fn rotation2<N: RealField + Copy>(rotation: &Rotation2<N>) -> HomogeneousMatrix2<N> {
    rotation.to_homogeneous()
}


pub fn scaling2<N: RealField + Copy>(scaling: &Scaling2<N>) -> HomogeneousMatrix2<N> {
    let mut matrix = HomogeneousMatrix2::<N>::identity();
    matrix[(0,0)] = scaling[0];
    matrix[(1,1)] = scaling[1];
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn test_translation2() {
        let res = translation2(&Translation2::<f32>::new(2., 3.));
        expect_that!(&res[(0,0)], close_to(1., 1e-6));
        expect_that!(&res[(0,1)], close_to(0., 1e-6));
        expect_that!(&res[(0,2)], close_to(2., 1e-6));
        expect_that!(&res[(1,0)], close_to(0., 1e-6));
        expect_that!(&res[(1,1)], close_to(1., 1e-6));
        expect_that!(&res[(1,2)], close_to(3., 1e-6));
        expect_that!(&res[(2,0)], close_to(0., 1e-6));
        expect_that!(&res[(2,1)], close_to(0., 1e-6));
        expect_that!(&res[(2,2)], close_to(1., 1e-6));
    }

    #[test]
    fn test_rotation2() {
        let res = rotation2(&Rotation2::<f32>::from_angle(1.));
        expect_that!(&res[(0,0)], close_to(0.540302, 1e-6));
        expect_that!(&res[(0,1)], close_to(-0.841471, 1e-6));
        expect_that!(&res[(0,2)], close_to(0., 1e-6));
        expect_that!(&res[(1,0)], close_to(0.841471, 1e-6));
        expect_that!(&res[(1,1)], close_to(0.540302, 1e-6));
        expect_that!(&res[(1,2)], close_to(0., 1e-6));
        expect_that!(&res[(2,0)], close_to(0., 1e-6));
        expect_that!(&res[(2,1)], close_to(0., 1e-6));
        expect_that!(&res[(2,2)], close_to(1., 1e-6));
    }

    #[test]
    fn test_scaling2() {
        let res = scaling2(&Scaling2::<f32>::new(2., 3.));
        expect_that!(&res[(0,0)], close_to(2., 1e-6));
        expect_that!(&res[(0,1)], close_to(0., 1e-6));
        expect_that!(&res[(0,2)], close_to(0., 1e-6));
        expect_that!(&res[(1,0)], close_to(0., 1e-6));
        expect_that!(&res[(1,1)], close_to(3., 1e-6));
        expect_that!(&res[(1,2)], close_to(0., 1e-6));
        expect_that!(&res[(2,0)], close_to(0., 1e-6));
        expect_that!(&res[(2,1)], close_to(0., 1e-6));
        expect_that!(&res[(2,2)], close_to(1., 1e-6));
    }
}