use std::default::Default;

pub use wgpu::Color as ColorF64;

#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const CYAN: Self = Self {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const MAGENTA: Self = Self {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl as_slice::AsSlice for Color {
    type Element = u8;
    fn as_slice(&self) -> &[Self::Element] {
        let pc: *const Color = self;
        let pc: *const u8 = pc as *const u8;
        unsafe { std::slice::from_raw_parts(pc, std::mem::size_of::<Color>()) }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ColorF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorF32 {
    pub const TRANSPARENT: Self = Self {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    pub const BLACK: Self = Self {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    pub const WHITE: Self = Self {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };
    pub const RED: Self = Self {
        r: 1.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    pub const GREEN: Self = Self {
        r: 0.,
        g: 1.,
        b: 0.,
        a: 1.,
    };
    pub const BLUE: Self = Self {
        r: 0.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
    pub const YELLOW: Self = Self {
        r: 1.,
        g: 1.,
        b: 0.,
        a: 1.,
    };
    pub const CYAN: Self = Self {
        r: 0.,
        g: 1.,
        b: 1.,
        a: 1.,
    };
    pub const MAGENTA: Self = Self {
        r: 1.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
}

impl Default for ColorF32 {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl as_slice::AsSlice for ColorF32 {
    type Element = f32;
    fn as_slice(&self) -> &[Self::Element] {
        let pc: *const ColorF32 = self;
        let pc: *const u8 = pc as *const u8;
        let data = unsafe { std::slice::from_raw_parts(pc, std::mem::size_of::<ColorF32>()) };
        bytemuck::cast_slice(&data)
    }
}

impl From<ColorF64> for Color {
    fn from(c: ColorF64) -> Self {
        const FACTOR: f64 = 255.;
        let r = num::clamp(c.r * FACTOR, 0., 255.) as u8;
        let g = num::clamp(c.g * FACTOR, 0., 255.) as u8;
        let b = num::clamp(c.b * FACTOR, 0., 255.) as u8;
        let a = num::clamp(c.a * FACTOR, 0., 255.) as u8;
        Self { r, g, b, a }
    }
}

impl From<Color> for ColorF64 {
    fn from(c: Color) -> Self {
        const FACTOR: f64 = 255.;
        let r = c.r as f64 / FACTOR;
        let g = c.g as f64 / FACTOR;
        let b = c.b as f64 / FACTOR;
        let a = c.a as f64 / FACTOR;
        Self { r, g, b, a }
    }
}

impl From<ColorF32> for Color {
    fn from(c: ColorF32) -> Self {
        const FACTOR: f32 = 255.;
        let r = num::clamp(c.r * FACTOR, 0., 255.) as u8;
        let g = num::clamp(c.g * FACTOR, 0., 255.) as u8;
        let b = num::clamp(c.b * FACTOR, 0., 255.) as u8;
        let a = num::clamp(c.a * FACTOR, 0., 255.) as u8;
        Self { r, g, b, a }
    }
}

impl From<Color> for ColorF32 {
    fn from(c: Color) -> Self {
        const FACTOR: f32 = 255.;
        let r = c.r as f32 / FACTOR;
        let g = c.g as f32 / FACTOR;
        let b = c.b as f32 / FACTOR;
        let a = c.a as f32 / FACTOR;
        Self { r, g, b, a }
    }
}

impl From<ColorF32> for ColorF64 {
    fn from(c: ColorF32) -> Self {
        Self {
            r: c.r as f64,
            g: c.g as f64,
            b: c.b as f64,
            a: c.a as f64,
        }
    }
}

impl From<ColorF64> for ColorF32 {
    fn from(c: ColorF64) -> Self {
        Self {
            r: c.r as f32,
            g: c.g as f32,
            b: c.b as f32,
            a: c.a as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use galvanic_assert::{matchers::*, *};

    #[test]
    fn color_to_color_f32_conversion() {
        let color = ColorF32::from(Color {
            r: 0,
            g: 255,
            b: 121,
            a: 217,
        });
        expect_that!(&color.r, close_to(0., 1e-16));
        expect_that!(&color.g, close_to(1., 1e-16));
        expect_that!(&color.b, close_to(121. / 255., 1e-16));
        expect_that!(&color.a, close_to(217. / 255., 1e-16));
    }

    #[test]
    fn color_f32_to_color_conversion() {
        let color = Color::from(ColorF32 {
            r: 0.,
            g: 1.,
            b: 0.3,
            a: 0.45,
        });
        expect_that!(&color.r, eq(0));
        expect_that!(&color.g, eq(255));
        expect_that!(&color.b, eq(76));
        expect_that!(&color.a, eq(114));
    }

    #[test]
    fn color_f32_to_color_conversion_limits() {
        let color = Color::from(ColorF32 {
            r: -1.,
            g: -2.,
            b: 2.,
            a: 30.,
        });
        expect_that!(&color.r, eq(0));
        expect_that!(&color.g, eq(0));
        expect_that!(&color.b, eq(255));
        expect_that!(&color.a, eq(255));
    }

    #[test]
    fn color_to_color_f64_conversion() {
        let color = ColorF64::from(Color {
            r: 0,
            g: 255,
            b: 121,
            a: 217,
        });
        expect_that!(&color.r, close_to(0., 1e-16));
        expect_that!(&color.g, close_to(1., 1e-16));
        expect_that!(&color.b, close_to(121. / 255., 1e-16));
        expect_that!(&color.a, close_to(217. / 255., 1e-16));
    }

    #[test]
    fn color_f64_to_color_conversion() {
        let color = Color::from(ColorF64 {
            r: 0.,
            g: 1.,
            b: 0.3,
            a: 0.45,
        });
        expect_that!(&color.r, eq(0));
        expect_that!(&color.g, eq(255));
        expect_that!(&color.b, eq(76));
        expect_that!(&color.a, eq(114));
    }

    #[test]
    fn color_f64_to_color_conversion_limits() {
        let color = Color::from(ColorF64 {
            r: -1.,
            g: -2.,
            b: 2.,
            a: 30.,
        });
        expect_that!(&color.r, eq(0));
        expect_that!(&color.g, eq(0));
        expect_that!(&color.b, eq(255));
        expect_that!(&color.a, eq(255));
    }

    #[test]
    fn color_f32_to_color_f64_conversion() {
        let color = ColorF64::from(ColorF32 {
            r: 0.,
            g: 1.,
            b: 0.25,
            a: 0.75,
        });
        expect_that!(&color.r, close_to(0., 1e-16));
        expect_that!(&color.g, close_to(1., 1e-16));
        expect_that!(&color.b, close_to(0.25, 1e-16));
        expect_that!(&color.a, close_to(0.75, 1e-16));
    }

    #[test]
    fn color_f64_to_color_f32_conversion() {
        let color = ColorF32::from(ColorF64 {
            r: 0.,
            g: 1.,
            b: 0.3,
            a: 0.45,
        });
        expect_that!(&color.r, close_to(0., 1e-16));
        expect_that!(&color.g, close_to(1., 1e-16));
        expect_that!(&color.b, close_to(0.3, 1e-16));
        expect_that!(&color.a, close_to(0.45, 1e-16));
    }
}
