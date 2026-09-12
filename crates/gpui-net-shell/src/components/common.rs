//! Shared validation helpers for component constructors and methods, mirroring
//! `component-shell`'s `compound/common.rs`.

pub(crate) fn nonempty_id(id: &str, component: &str) -> Result<String, String> {
    if id.trim().is_empty() {
        return Err(format!("{component}(id) expects a nonempty string id"));
    }
    Ok(id.to_owned())
}

pub(crate) fn finite_f32(value: f64, label: &str) -> Result<f32, String> {
    if !value.is_finite() || value < f32::MIN as f64 || value > f32::MAX as f64 {
        return Err(format!(
            "{label} expects a finite number representable as f32"
        ));
    }
    Ok(value as f32)
}

/// An exactly representable positive integer.
///
/// On 64-bit targets `usize::MAX as f64` rounds to 2^64, hence the exclusive
/// upper bound before converting with `as`.
pub(crate) fn positive_usize(value: f64, label: &str) -> Result<usize, String> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value >= usize::MAX as f64 {
        return Err(format!("{label} expects a positive integer"));
    }
    let value = value as usize;
    if value == 0 {
        return Err(format!("{label} expects a positive integer"));
    }
    Ok(value)
}

/// An exactly representable non-negative integer.
pub(crate) fn nonnegative_usize(value: f64, label: &str) -> Result<usize, String> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value >= usize::MAX as f64 {
        return Err(format!("{label} expects a non-negative integer"));
    }
    Ok(value as usize)
}

/// An exactly representable positive `u16`.
pub(crate) fn positive_u16(value: f64, label: &str) -> Result<u16, String> {
    if !value.is_finite() || value < 1.0 || value.fract() != 0.0 || value > u16::MAX as f64 {
        return Err(format!(
            "{label} expects an exactly representable positive integer"
        ));
    }
    Ok(value as u16)
}

/// A finite non-negative `f32`.
pub(crate) fn nonnegative_f32(value: f64, label: &str) -> Result<f32, String> {
    if !value.is_finite() || value < 0.0 || value > f32::MAX as f64 {
        return Err(format!("{label} expects a nonnegative finite number"));
    }
    Ok(value as f32)
}
