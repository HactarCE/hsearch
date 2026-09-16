use crate::*;

/// Returns the new orientation for a ridge in stage 1.
///
/// - `r` = Rotation matrix to apply
/// - `v` = Old position
/// - `o` = Old orientation bits (just lowest 2 bits)
pub fn s1_ro(r: Mat4, v: Vec4, o: u8) -> u8 {
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

/// Returns the new orientation for an edge in stage 3.
///
/// - `r` = Rotation matrix to apply
/// - `v` = Old position
/// - `o` = Old orientation bits (just lowest 2 bits)
pub fn s3_eo(r: Mat4, v: Vec4, o: u8) -> u8 {
    match o {
        0 => o,
        _ => {
            let old = v.nonzero_axes().nth(3 - o as usize).unwrap();
            let new = old.transform_by(r);
            3 - (r * v).nonzero_axes().position(|a| a == new).unwrap() as u8
        }
    }
}
