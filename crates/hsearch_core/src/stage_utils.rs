use crate::*;

/// Returns an updated ridge orientation relative to the first axis on which it
/// has nonzero coordinate. I.e., X axis if it is on `R`/`L`, otherwise Y axis
/// if it is on `U`/`D`, otherwise `Z` axis.
///
/// - `r` = Rotation matrix to apply
/// - `v` = Old position
/// - `o` = Old orientation bits (just lowest 2 bits)
pub fn xyz_ro(r: Mat4, v: Vec4, o: u8) -> u8 {
    /// Canonical axis order for determining ridge orientation.
    const AXIS_ORDER: [Axis; 4] = [X, Y, Z, W];

    match o {
        0b00 | 0b11 => o,
        0b10 | 0b01 => {
            let old_axis = v.unwrap_first_nonzero_axis(AXIS_ORDER);
            let new_axis = (r * v).unwrap_first_nonzero_axis(AXIS_ORDER);
            if old_axis.transform_by(r) == new_axis {
                o
            } else {
                o ^ 0b11
            }
        }
        _ => unreachable!(),
    }
}

/// Returns an initial edge orientation relative to the X axis.
pub fn rl_init_eo(init: Vec4, att: Mat4) -> u64 {
    let o = if init[X] == 0 { 0 } else { 3 };
    rl_eo(att, init, o) as u64
}

/// Returns an updated edge orientation relative to the X axis.
///
/// - `r` = Rotation matrix to apply
/// - `v` = Old position
/// - `o` = Old orientation bits (just lowest 2 bits)
pub fn rl_eo(r: Mat4, v: Vec4, o: u8) -> u8 {
    match o {
        0 => o,
        _ => {
            let old = v.nonzero_axes().nth(3 - o as usize).unwrap();
            let new = old.transform_by(r);
            3 - (r * v).nonzero_axes().position(|a| a == new).unwrap() as u8
        }
    }
}
