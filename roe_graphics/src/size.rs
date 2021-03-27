use num_traits::identities::Zero;
use std::cmp::PartialOrd;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Size<T: Copy + Zero + PartialOrd> {
    width: T,
    height: T,
}

impl<T: Copy + Zero + PartialOrd> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        assert!(
            width >= T::zero() && height >= T::zero(),
            "A negative size is invalid"
        );
        Self { width, height }
    }

    pub fn width(&self) -> T {
        self.width
    }

    pub fn set_width(&mut self, value: T) {
        assert!(value >= T::zero(), "A negative size is invalid");
        self.width = value;
    }

    pub fn height(&self) -> T {
        self.height
    }

    pub fn set_height(&mut self, value: T) {
        assert!(value >= T::zero(), "A negative size is invalid");
        self.height = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    #[serial_test::serial]
    fn creation() {
        let size = Size::<f32>::new(3., 4.);
        expect_that!(&size.width(), eq(3.));
        expect_that!(&size.height(), eq(4.));
    }

    #[test]
    #[serial_test::serial]
    fn zero() {
        let size = Size::<f32>::new(0., 0.);
        expect_that!(&size.width(), eq(0.));
        expect_that!(&size.height(), eq(0.));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "A negative size is invalid")]
    fn creation_failure_invalid_width() {
        let _size = Size::<f32>::new(-3., 4.);
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "A negative size is invalid")]
    fn creation_failure_invalid_height() {
        let _size = Size::<f32>::new(3., -4.);
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "A negative size is invalid")]
    fn creation_failure_invalid_width_and_height() {
        let _size = Size::<f32>::new(-3., -4.);
    }

    #[test]
    #[serial_test::serial]
    fn set_width() {
        let mut size = Size::<f32>::new(0., 0.);
        expect_that!(&size.width(), eq(0.));
        expect_that!(&size.height(), eq(0.));
        size.set_width(2.);
        expect_that!(&size.width(), eq(2.));
        expect_that!(&size.height(), eq(0.));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "A negative size is invalid")]
    fn set_width_invalid_value() {
        let mut size = Size::<f32>::new(0., 0.);
        size.set_width(-2.);
    }

    #[test]
    #[serial_test::serial]
    fn set_height() {
        let mut size = Size::<f32>::new(0., 0.);
        expect_that!(&size.width(), eq(0.));
        expect_that!(&size.height(), eq(0.));
        size.set_height(3.);
        expect_that!(&size.width(), eq(0.));
        expect_that!(&size.height(), eq(3.));
    }

    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "A negative size is invalid")]
    fn set_height_invalid_value() {
        let mut size = Size::<f32>::new(0., 0.);
        size.set_height(-2.);
    }
}
