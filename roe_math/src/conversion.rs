use super::{geometry2, geometry3, RealField};

pub fn transform2_to_transform3<N: RealField + Copy>(
    transform2: &geometry2::Transform<N>,
) -> geometry3::Transform<N> {
    let mut out = geometry3::Transform::<N>::identity();
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
    fn test_transform2_to_transform3() {
        let transform2 = geometry2::Transform::new(1., 2., 3., 4., 5., 6., 7., 8., 9.);
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
