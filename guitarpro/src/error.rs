use fraction::ToPrimitive;
use thiserror::Error;

/// Error type for Guitar Pro file parsing
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GpError {
    /// Reached end of binary data unexpectedly
    #[error("Unexpected EOF at offset {offset}, needed {needed} bytes")]
    UnexpectedEof { offset: usize, needed: usize },

    /// Invalid enum/flag value encountered during parsing
    #[error("Invalid value {value} for {context}")]
    InvalidValue { context: &'static str, value: i64 },

    /// String decoding failure
    #[error("String decode error at offset {offset}")]
    StringDecode { offset: usize },

    /// ZIP, XML, or format-level errors from GP6/GP7 parsing
    #[error("Format error: {0}")]
    FormatError(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Type conversion failed (value out of range for target type)
    #[error("Type conversion failed for {context}: {value} out of range")]
    TypeConversion { context: &'static str, value: i64 },

    /// Required state not set (e.g., current_track not initialized)
    #[error("Required state '{field}' not set")]
    MissingState { field: &'static str },

    /// Write operation error
    #[error("Write error: {0}")]
    WriteError(String),
}

/// Convenience type alias
pub type GpResult<T> = Result<T, GpError>;

impl From<String> for GpError {
    fn from(msg: String) -> Self {
        GpError::FormatError(msg)
    }
}

/// Extension trait for numeric conversions with context-aware errors.
/// Use this instead of `ToPrimitive::to_*().unwrap()` to get proper error messages.
pub trait ToPrimitiveGp {
    fn to_u8_gp(&self, ctx: &'static str) -> GpResult<u8>;
    fn to_i8_gp(&self, ctx: &'static str) -> GpResult<i8>;
    fn to_i16_gp(&self, ctx: &'static str) -> GpResult<i16>;
    fn to_u16_gp(&self, ctx: &'static str) -> GpResult<u16>;
    fn to_i32_gp(&self, ctx: &'static str) -> GpResult<i32>;
    fn to_i64_gp(&self, ctx: &'static str) -> GpResult<i64>;
    fn to_usize_gp(&self, ctx: &'static str) -> GpResult<usize>;
    fn to_f32_gp(&self, ctx: &'static str) -> GpResult<f32>;
    fn to_f64_gp(&self, ctx: &'static str) -> GpResult<f64>;
}

macro_rules! impl_to_primitive_gp {
    ($($t:ty),*) => {
        $(
            impl ToPrimitiveGp for $t {
                fn to_u8_gp(&self, ctx: &'static str) -> GpResult<u8> {
                    self.to_u8().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
                fn to_i8_gp(&self, ctx: &'static str) -> GpResult<i8> {
                    self.to_i8().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
                fn to_i16_gp(&self, ctx: &'static str) -> GpResult<i16> {
                    self.to_i16().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
                fn to_u16_gp(&self, ctx: &'static str) -> GpResult<u16> {
                    self.to_u16().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
                fn to_i32_gp(&self, ctx: &'static str) -> GpResult<i32> {
                    self.to_i32().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
                fn to_i64_gp(&self, ctx: &'static str) -> GpResult<i64> {
                    self.to_i64().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
                fn to_usize_gp(&self, ctx: &'static str) -> GpResult<usize> {
                    self.to_usize().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
                fn to_f32_gp(&self, ctx: &'static str) -> GpResult<f32> {
                    self.to_f32().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
                fn to_f64_gp(&self, ctx: &'static str) -> GpResult<f64> {
                    self.to_f64().ok_or(GpError::TypeConversion { context: ctx, value: *self as i64 })
                }
            }
        )*
    };
}

impl_to_primitive_gp!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

// Special implementation for f32 and f64 (can't cast to i64 directly)
impl ToPrimitiveGp for f32 {
    fn to_u8_gp(&self, ctx: &'static str) -> GpResult<u8> {
        self.to_u8().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_i8_gp(&self, ctx: &'static str) -> GpResult<i8> {
        self.to_i8().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_i16_gp(&self, ctx: &'static str) -> GpResult<i16> {
        self.to_i16().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_u16_gp(&self, ctx: &'static str) -> GpResult<u16> {
        self.to_u16().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_i32_gp(&self, ctx: &'static str) -> GpResult<i32> {
        self.to_i32().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_i64_gp(&self, ctx: &'static str) -> GpResult<i64> {
        self.to_i64().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_usize_gp(&self, ctx: &'static str) -> GpResult<usize> {
        self.to_usize().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_f32_gp(&self, _ctx: &'static str) -> GpResult<f32> {
        Ok(*self)
    }
    fn to_f64_gp(&self, ctx: &'static str) -> GpResult<f64> {
        self.to_f64().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
}

impl ToPrimitiveGp for f64 {
    fn to_u8_gp(&self, ctx: &'static str) -> GpResult<u8> {
        self.to_u8().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_i8_gp(&self, ctx: &'static str) -> GpResult<i8> {
        self.to_i8().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_i16_gp(&self, ctx: &'static str) -> GpResult<i16> {
        self.to_i16().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_u16_gp(&self, ctx: &'static str) -> GpResult<u16> {
        self.to_u16().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_i32_gp(&self, ctx: &'static str) -> GpResult<i32> {
        self.to_i32().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_i64_gp(&self, ctx: &'static str) -> GpResult<i64> {
        self.to_i64().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_usize_gp(&self, ctx: &'static str) -> GpResult<usize> {
        self.to_usize().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_f32_gp(&self, ctx: &'static str) -> GpResult<f32> {
        self.to_f32().ok_or(GpError::TypeConversion {
            context: ctx,
            value: *self as i64,
        })
    }
    fn to_f64_gp(&self, _ctx: &'static str) -> GpResult<f64> {
        Ok(*self)
    }
}
