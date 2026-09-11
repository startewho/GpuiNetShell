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
