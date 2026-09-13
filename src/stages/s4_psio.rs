use crate::XyRot;

use super::*;

/// Stage 4: `P`-separation + `I`/`O` edge & corner orientation
///
/// ## Invariants
///
/// - All `I`/`O` ridges must remain oriented.
/// - All pieces outside the `F` facet (`[-1, -1, 1, -1]..=[1, 1, 1, 1]`) and
///   `UO` (`[-1, -1, 1, 1]..=[1, 1, 1, 1]`) ridge must remain oriented &
///   `P`-separated.
///
/// ## Move set
///
/// 33 twists are allowed:
///
/// - All `F` twists (23 twists)
/// - `IF`, `IF2`, and `IB` (3 twists)
/// - `OF`, `OF2`, and `OB` (3 twists)
/// - Certain `O` twists, followed by an implicit rotation to preserve
///   invariants:
///     - `OR` (1 twist)
///     - `OUFR`, and `OUFL` (2 twists)
///     - `OUF` (1 twist)
///
/// ## Target
///
/// - All `P` pieces are in `P`.
/// - All `I`/`O` pieces are in `I`/`O` and are `I`/`O`-oriented.
///
/// This target has 1 possible orientation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage4 {
    /// For each OU/F corner, 2 bits indicating the axis containing its I/O
    /// sticker:
    ///
    /// - `00` = X
    /// - `01` = Y
    /// - `10` = Z
    /// - `11` = W
    ///
    /// For each OU/F edge, 2 bits indicating one of the following cases:
    ///
    /// - `00` = belongs in P slice, any orientation
    /// - `01` = belongs in I/O, good orientation
    /// - `10` = belongs in I/O, bad orientation 1
    /// - `11` = belongs in I/O, bad orientation 2
    ///
    /// For each OU/F ridge, 1 bit indicating one of the following cases:
    ///
    /// - `0` = belongs in the P slice
    /// - `1` = belongs in I/O
    ///
    /// Total: 6 corners (20 bits), 15 edges (30 bits), 7 ridges (7 bits)
    pub bits: u64, // (u20, u30, u7)
}

impl Default for Stage4 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl Stage4 {
    pub const SOLVED: Self = Self {
        bits: 0x01855550055fffff,
    };

    pub fn is_target_solved(self, target: Self) -> bool {
        self.bits == target.bits
    }

    /// Returns `Err(())` if the twist is disallowed, `Ok(None)` if the twist is
    /// allowed and has no full-puzzle rotation, or `Ok(Some(rot))` if the twist
    /// is allowed and `rot` should be applied to all pieces after the twist.
    ///
    /// This method is relatively slow and should not be called on a hot path.
    #[cfg(test)]
    fn rotation_applied_after_twist(t: TwistData) -> Result<Option<Mat4>, ()> {
        // The region of unsolved pieces can be split into two blocks:
        // - 3x3x3x1 (the entire F cell)
        // - 3x2x1x1 (OU, not including F)
        match t.facet {
            // F twists rotate the 3x3x3x1, which preserves the shape of the
            // unsolved pieces.
            F => Ok(None), // 23 twists (F*)
            // I twists in the XY plane preserve all the invariants.
            I if F.transform_by(t.rot) == F => Ok(None), // 3 twists (IF, IF2, IB)
            // O twists may move the 3x2x1x1, changing the shape of the unsolved
            // pieces. We must apply a whole-puzzle rotation afterward to keep
            // the extra 3x2x1x1 block in OU.
            O => {
                let new_f = F.transform_by(t.rot);
                let new_u = U.transform_by(t.rot);
                if new_f == F {
                    Ok(Some(t.rot.inv())) // 3 twists (OF, OF2, OB)
                } else if new_u == F {
                    match new_f {
                        U => Ok(None),                         // 1 twist (OUF)
                        R | L => Ok(Some(new_f.mat4_to(U))),   // 2 twists (OUFR, OUFL)
                        D => Ok(Some(Mat4::rot(X, Y).pow(2))), // 1 twist (OR)
                        _ => Err(()),
                    }
                } else {
                    Err(())
                }
            }

            // total: 23+3+3+3+1 = 33 allowed twists

            // Other moves are not allowed.
            _ => Err(()),
        }
    }

    pub const TWISTS: [Twist; 33] = [
        Twist(4),
        Twist(6),
        Twist(12),
        Twist(20),
        Twist(28),
        Twist(36),
        Twist(44),
        Twist(52),
        Twist(60),
        Twist(68),
        Twist(70),
        Twist(76),
        Twist(84),
        Twist(92),
        Twist(100),
        Twist(102),
        Twist(103),
        Twist(108),
        Twist(116),
        Twist(118),
        Twist(124),
        Twist(132),
        Twist(140),
        Twist(148),
        Twist(150),
        Twist(156),
        Twist(164),
        Twist(172),
        Twist(174),
        Twist(175),
        Twist(180),
        Twist(182),
        Twist(183),
    ];
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    use crate::{lut_gen::*, util::collect_bits};

    fn filter_pos(v: Vec4) -> bool {
        v[Z] == 1 || (v[W] == 1 && v[Y] == 1)
    }

    fn corners() -> impl Iterator<Item = Vec4> {
        PieceType::Corner.iter().filter(|&v| filter_pos(v))
    }
    fn edges() -> impl Iterator<Item = Vec4> {
        PieceType::Edge.iter().filter(|&v| filter_pos(v))
    }
    fn ridges() -> impl Iterator<Item = Vec4> {
        PieceType::Ridge.iter().filter(|&v| filter_pos(v))
    }

    #[test]
    fn print_stage4_constants() {
        println!("pub const SOLVED: Self = Self {{");
        let m = collect_bits(itertools::chain!(
            corners().flat_map(|_| [true, true]),
            edges().flat_map(|v| [v[W] != 0, false]),
            ridges().map(|v| v[W] != 0)
        ));
        println!("    bits: 0x{m:016x},");
        println!("}};");
        println!();
    }

    #[test]
    fn lutgen_stage4_implicit_rotation() {
        let mut twists_with_nontrivial_rotation = [vec![], vec![], vec![]];
        for t in Twist::iter() {
            let Ok(Some(m)) = Stage4::rotation_applied_after_twist(t.data()) else {
                continue;
            };
            let rot = XyRot::from_mat4(m);
            if !rot.is_ident() {
                twists_with_nontrivial_rotation[rot.pow() as usize - 1].push(t);
            }
        }

        println!("match twist.to_index() {{");
        for i in 0..3 {
            let pow = i + 1;
            let twists = &twists_with_nontrivial_rotation[i];
            println!(
                "    {} => XyRot::from_pow({pow}), // {}",
                twists.iter().map(|t| t.to_index()).join(" | "),
                twists.iter().map(|t| t.data()).join(", "),
            );
        }
        println!("    _ => XyRot::IDENT, // all others");
        println!("}}")
    }

    #[test]
    fn lutgen_stage4() {
        println!();
        println!("let Self {{ bits }} = self;");

        fn apply_twist_with_rotation_if_needed(t: TwistData, p: Vec4) -> Option<Vec4> {
            let p = if t.affects(p) { t.rot * p } else { p };
            if let Some(extra_rot) = Stage4::rotation_applied_after_twist(t).ok()? {
                Some(extra_rot * p)
            } else {
                Some(p)
            }
        }

        let lut1 = OrientationLut::with_action(corners().chain(edges()), 4, |t, v, o| {
            let mut r = if t.affects(v) { t.rot } else { IDENT };
            if let Some(extra_rot) = Stage4::rotation_applied_after_twist(t).ok()? {
                r = extra_rot * r;
            }
            Some(if v.taxicab_norm() == 3 {
                // edge
                match o {
                    0b00 => o, // P slice
                    _ => {
                        let old_io_sticker_axis = v.nonzero_axes()[3 - o as usize];
                        let new_io_sticker_axis = old_io_sticker_axis.transform_by(r);
                        let new_index = (r * v)
                            .nonzero_axes()
                            .iter()
                            .position(|&ax| ax == new_io_sticker_axis)
                            .unwrap();
                        3 - new_index as u8
                    }
                }
            } else {
                // corner
                Axis::from_u8(o).transform_by(r) as u8
            })
        });
        println!("let bits = {};", lut1.to_rust_code(64, 0, 2, "bits"));

        let lut2 = PermutationLut::with_action(
            corners().chain(edges()),
            apply_twist_with_rotation_if_needed,
        );
        println!("let bits = {};", lut2.to_rust_code(64, 0, 2, "bits"));

        let lut3 = PermutationLut::with_action(ridges(), apply_twist_with_rotation_if_needed);
        println!("let bits = {};", lut3.to_rust_code(64, 50, 1, "bits"));

        println!("Self {{ bits }}");

        println!();
        let twists = lut2.allowed_twists();
        println!(
            "pub const TWISTS: [Twist; {}] = {:?};",
            twists.len(),
            twists,
        );
        println!();
    }
}

impl StageKeyU64 for Stage4 {
    fn key(self) -> u64 {
        self.bits
    }
    const PRUNING_MAP_TWISTS: &[Twist] = &Self::TWISTS;
}

impl Stage for Stage4 {
    fn implicit_rotation_after_twist(twist: Twist) -> XyRot {
        match twist.to_index() {
            70 | 102 => XyRot::from_pow(1),  // OB, ORUF
            150 | 174 => XyRot::from_pow(2), // OR, OF2
            6 | 182 => XyRot::from_pow(3),   // ORDB, OF
            _ => XyRot::IDENT,               // all others
        }
    }

    fn from_state(state: SimplePuzzleSim) -> Self {
        for &(pos, att) in &state.pieces {
            let p = att * pos;
            if !((p[W] == 1 && p[Y] == 1) || p[Z] == 1) && p.taxicab_norm() >= 3 {
                assert_eq!(W.transform_by(att), W, "pos is bad: {}", p);
            }
        }
        Self {
            bits: state.to_bits(
                2,
                &[PieceType::Corner, PieceType::Edge],
                |v| (v[W] == 1 && v[Y] == 1) || v[Z] == 1,
                |init, att| {
                    if init.taxicab_norm() == 3 {
                        // edge
                        if init[W] == 0 {
                            0
                        } else {
                            let old_io_sticker_axis = W;
                            let new_io_sticker_axis = old_io_sticker_axis.transform_by(att);
                            let new_index = (att * init)
                                .nonzero_axes()
                                .iter()
                                .position(|&ax| ax == new_io_sticker_axis)
                                .unwrap();
                            3 - new_index as u64
                        }
                    } else {
                        // corner
                        W.transform_by(att) as u64
                    }
                },
            ) | (state.to_bits(
                1,
                &[PieceType::Ridge],
                |v| (v[W] == 1 && v[Y] == 1) || v[Z] == 1,
                |init, _att| (init[W] != 0) as u64,
            ) << 50),
        }
    }

    fn do_twist(self, twist: Twist) -> Self {
        let Self { bits } = self;
        let bits = crate::lut::update_orientations_u64(
            bits,
            [
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0xFF0FF, 0xFFFCF3FAA3C55F55, 0x3AC0FFEBFF0FF],
                [0, 0, 0],
                [0x55, 0xFFFFFFFFFFF000FF, 0x10415500FFF55],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0xFF0FF, 0xFFFCF3FAA3C55F55, 0x3AC0FFEBFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0xFF0FF, 0xFFFDF7FAA7D00F00, 0x3AC0FFEBFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x55055, 0xFFFFFFFFFFFFFFFF, 0x550055055],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x55055, 0xFFFF5FF00D7AAFAA, 0x2F80FFBEFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x55055, 0xFFFF5FF00D7AAFAA, 0x2F80FFBEFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x55055, 0xFFFF5FF00D7AAFAA, 0x2F80FFBEFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0xFF0FF, 0xFFFDF7FAA7D00F00, 0x3AC0FFEBFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x55055, 0xFFFFFFFFFFFFFFFF, 0x550055055],
                [0, 0, 0],
                [0x55, 0xFFFFFFFFFFFFFFFF, 0x550000055],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x0, 0xFFFF0FF00C3FFFFF, 0x2F80FFBEAA0AA],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0xFF0FF, 0xFFFCF3FAA3C55F55, 0x3AC0FFEBFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0xFF0FF, 0xFFFCF3FAA3C55F55, 0x3AC0FFEBFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0xFF0FF, 0xFFFDF7FAA7D00F00, 0x3AC0FFEBFF0FF],
                [0, 0, 0],
                [0x55, 0xFFFFFFFFFFF000FF, 0x10415500FFF55],
                [0x55, 0xFFFFFFFFFFFFFFFF, 0x55],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x55055, 0xFFFFFFFFFFFFFFFF, 0x550055055],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x0, 0xFFFF0FF00C3FFFFF, 0x2F80FFBEAA0AA],
                [0, 0, 0],
                [0x0, 0xFFFFFFFFFFF000FF, 0x10410000FFF00],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x0, 0xFFFFFFFFFFFFFFFF, 0x0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x0, 0xFFFFFFFFFFFFFFFF, 0x0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0xFF0FF, 0xFFFDF7FAA7D00F00, 0x3AC0FFEBFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x55055, 0xFFFFFFFFFFFFFFFF, 0x550055055],
                [0, 0, 0],
                [0x0, 0xFFFFFFFFFFF000FF, 0x10410000FFF00],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x55055, 0xFFFF5FF00D7AAFAA, 0x2F80FFBEFF0FF],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x0, 0xFFFF0FF00C3FFFFF, 0x2F80FFBEAA0AA],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x0, 0xFFFFFFFFFFFFFFFF, 0x0],
                [0, 0, 0],
                [0x0, 0xFFFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFFFF, 0x0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0x0, 0xFFFF0FF00C3FFFFF, 0x2F80FFBEAA0AA],
                [0, 0, 0],
                [0x55, 0xFFFFFFFFFFFFFFFF, 0x550000055],
                [0x55, 0xFFFFFFFFFFFFFFFF, 0x55],
            ][twist.to_index()],
        );
        let bits = apply_permutation_lut!(u64, bits, twist, [
            4 => [(&0xFFFC03F0000C0F03<<0)|(&0xC0000000C000<<2)|(&0xC000000<<4)|(&0x3000C0<<8)|(&0xC<<10)|(&0xC00000000<<12)|(&0xC0000000<<14)|(&0x3000000<<18)|(&0x300000000000<<46)|(&0x3000000000000<<50)|(&0xC0000030000<<54)|(&0x300003000<<56)|(&0x30000000<<58)|(&0xC00030<<62)],
            6 => [(&0xFFFF000000000000<<0)|(&0x300330C30<<2)|(&0x30030C00003<<4)|(&0x3000000300<<6)|(&0xC000000000<<8)|(&0xC00000000000<<56)|(&0xC000000C000<<58)|(&0x300C030000C0<<60)|(&0xCC0C300C<<62)],
            12 => [(&0xFFFC03F000030F0C<<0)|(&0xC000000<<2)|(&0x300000000000<<4)|(&0x3000<<6)|(&0x30<<8)|(&0x300000<<10)|(&0x300000000<<12)|(&0x3<<14)|(&0x30000000<<18)|(&0xC00000<<20)|(&0xC00000000000<<44)|(&0x3000000000000<<48)|(&0xC0000<<50)|(&0xC00000000<<52)|(&0xC000000C000<<56)|(&0xC00000C0<<58)|(&0x3000000<<60)],
            20 => [(&0xFFFC03F000000F00<<0)|(&0xC00033<<2)|(&0xC000000<<8)|(&0x300000<<10)|(&0xC00000CC<<12)|(&0xC00000000<<14)|(&0x3000000<<22)|(&0x300000000000<<42)|(&0x3000000000000<<48)|(&0xC0000000000<<50)|(&0x33000<<52)|(&0x30000000<<56)|(&0x300000000<<58)|(&0xC000000CC000<<62)],
            28 => [(&0xFFFC03F000000F00<<0)|(&0xC0300330030<<2)|(&0x300030C03003<<4)|(&0xC00C030C00C0<<60)|(&0x30000CC00C00C<<62)],
            36 => [(&0xFFFC03F00000CF30<<0)|(&0xC0003000000<<4)|(&0x3000<<6)|(&0xC0000C<<10)|(&0xC0000000<<12)|(&0x3<<16)|(&0x30000000<<20)|(&0x300000<<24)|(&0x3000000000000<<40)|(&0xC000000C0000<<48)|(&0xC00000000<<50)|(&0x300000030000<<54)|(&0x3000000C0<<58)|(&0xC000000<<60)],
            44 => [(&0xFFFC03F000003FC0<<0)|(&0xC000000C000<<2)|(&0xC00000<<8)|(&0x3000000<<10)|(&0x30000003<<14)|(&0xC<<16)|(&0xC0000000<<18)|(&0x300000<<26)|(&0x3000000000000<<38)|(&0x300000030000<<48)|(&0xC000000C0000<<50)|(&0x300000000<<52)|(&0xC00000000<<56)|(&0xC000030<<62)],
            52 => [(&0xFFFC03F0000C0F03<<0)|(&0x30000C<<2)|(&0xC00000<<6)|(&0x3000030<<8)|(&0x3000000C0<<10)|(&0xC00000000<<14)|(&0xC000000<<18)|(&0xC0000000000<<46)|(&0x300000000000<<50)|(&0xC00000000000<<52)|(&0x3000<<54)|(&0x3000C000<<56)|(&0xC0000000<<60)|(&0x3000000030000<<62)],
            60 => [(&0xFFFCC3F000C00F00<<0)|(&0xC000000<<2)|(&0xC003<<4)|(&0xC0<<6)|(&0xC00000000<<8)|(&0x300000<<12)|(&0xC<<14)|(&0xC0000000<<18)|(&0x3000000<<20)|(&0x300000000000<<44)|(&0x3000000000000<<46)|(&0x30000<<50)|(&0x300000000<<52)|(&0xC0000000000<<56)|(&0x3000<<58)|(&0xC0030<<60)|(&0x30000000<<62)],
            68 => [(&0xFFFC03FC30000F00<<0)|(&0xC0000000<<2)|(&0x30<<10)|(&0xC3<<12)|(&0xC<<14)|(&0xCC00000<<20)|(&0x3300000<<24)|(&0x3300000000000<<40)|(&0xCC0000000000<<44)|(&0x30000<<50)|(&0xC3000<<52)|(&0xC000<<54)|(&0x300000000<<62)],
            70 => [(&0xFFFFFFF0000FFF00<<0)|(&0x33000003<<2)|(&0xC030000C<<4)|(&0x30C000030<<60)|(&0xC00C000C0<<62)],
            76 => [(&0xFFFC03F000000F00<<0)|(&0xF000<<4)|(&0xC0003C00000<<6)|(&0xF<<12)|(&0xF0000000<<16)|(&0x300000<<22)|(&0x3000000000000<<42)|(&0xF000000F0000<<52)|(&0xF00000000<<54)|(&0xC000000<<58)|(&0xF0<<60)],
            84 => [(&0xFFFC03F000003FC0<<0)|(&0x300000C<<2)|(&0xC000000<<8)|(&0x300000<<12)|(&0x300000030<<14)|(&0x30000003<<16)|(&0xC00000<<26)|(&0xC00000000000<<38)|(&0x3000000000000<<46)|(&0xC0000<<48)|(&0xC000000C000<<50)|(&0xC00000000<<54)|(&0xC0000000<<56)|(&0x300000030000<<62)],
            92 => [(&0xFFFC03F00000CF30<<0)|(&0xC00000<<4)|(&0xC000003<<6)|(&0xC000000C0<<10)|(&0x300000<<14)|(&0xC000000C<<16)|(&0x3000000<<24)|(&0x300000000000<<40)|(&0x3000000000000<<44)|(&0x30000<<48)|(&0xC0000000000<<52)|(&0x300003000<<54)|(&0xC0000<<58)|(&0xC00030000000<<60)],
            100 => [(&0xFFFC33F003000F00<<0)|(&0xC00300C<<4)|(&0x300000030<<10)|(&0x300000<<14)|(&0x3<<18)|(&0x30000000<<20)|(&0xC00000<<24)|(&0xC00000000000<<40)|(&0x3000000000000<<44)|(&0xC0000<<46)|(&0xC00000000<<50)|(&0xC000000C000<<54)|(&0xC00300C0<<60)],
            102 => [(&0xFFFF000000000000<<0)|(&0x33030C03<<2)|(&0x300C030000C<<4)|(&0x3000000300<<6)|(&0xC000000000<<8)|(&0xC00000000000<<56)|(&0xC000000C000<<58)|(&0x30030C000030<<60)|(&0xC00CC30C0<<62)],
            103 => [(&0xFFFFFFFFF00FFF00<<0)|(&0x3000003<<2)|(&0x30000C<<4)|(&0xC000030<<60)|(&0xC000C0<<62)],
            108 => [(&0xFFFC03F3C0000F00<<0)|(&0x300000C0<<6)|(&0x3C<<12)|(&0xF000003<<18)|(&0xF00000<<26)|(&0x3C00000000000<<38)|(&0x3C00000C0000<<46)|(&0x3C000<<52)|(&0xC00003000<<58)],
            116 => [(&0xFFFC0FF00C000F00<<0)|(&0x3030<<2)|(&0x3000000<<8)|(&0xC00000<<12)|(&0xC000000C<<14)|(&0x30000003<<18)|(&0x300000<<28)|(&0x3000000000000<<36)|(&0xC000000C0000<<46)|(&0x300000030000<<50)|(&0xC00000000<<52)|(&0x300000000<<56)|(&0xC0C0<<62)],
            118 => [(&0xFFFF000FFFF000FF<<0)|(&0x30C00<<2)|(&0x30000000000<<4)|(&0x3000000300<<6)|(&0xC000000000<<8)|(&0xC00000000000<<56)|(&0xC000000C000<<58)|(&0x300000000000<<60)|(&0xC3000<<62)],
            124 => [(&0xFFFC03F000000F00<<0)|(&0x330000000<<2)|(&0xCC<<10)|(&0x33<<14)|(&0x3000000<<20)|(&0xC300000<<22)|(&0xC00000<<24)|(&0xC00000000000<<40)|(&0x30C0000000000<<42)|(&0x300000000000<<44)|(&0xCC000<<50)|(&0x33000<<54)|(&0xCC0000000<<62)],
            132 => [(&0xFFFC03F000000F00<<0)|(&0x3000C0C0C00C<<2)|(&0xC0030303003<<6)|(&0x3000C0C0C00C0<<58)|(&0xC00303030030<<62)],
            140 => [(&0xFFFC03F000000F00<<0)|(&0x300000033000<<2)|(&0xC000000<<6)|(&0x300000<<8)|(&0x33<<12)|(&0x30000000<<14)|(&0x300000000<<16)|(&0xC00000<<22)|(&0xC00000000000<<42)|(&0x3000000000000<<50)|(&0xC00000CC000<<52)|(&0xC0000000<<54)|(&0xC00000000<<56)|(&0x30000CC<<62)],
            148 => [(&0xFFFC03F000000F00<<0)|(&0xC00033003003<<2)|(&0xC00C030C00C<<4)|(&0x300030C030030<<60)|(&0x300C00CC00C0<<62)],
            150 => [(&0xFFFF000000000000<<0)|(&0xC0C30C0C<<2)|(&0x30000000000<<4)|(&0x3030300303<<6)|(&0xC000000000<<8)|(&0xC00000000000<<56)|(&0xC0C0C00C0C0<<58)|(&0x300000000000<<60)|(&0x3030C3030<<62)],
            156 => [(&0xFFFC03F000030F0C<<0)|(&0x300000<<4)|(&0x3000003<<6)|(&0xC000000C0<<8)|(&0xC00000<<12)|(&0x30<<14)|(&0x300000000<<16)|(&0xC000000<<20)|(&0xC0000000000<<44)|(&0xC00000000000<<46)|(&0xC000<<50)|(&0x300000000000<<52)|(&0xC0000000<<54)|(&0x3000<<56)|(&0xC0000<<58)|(&0x3000000000000<<60)|(&0x30000000<<62)],
            164 => [(&0xFFFF03F000300F00<<0)|(&0x30003<<2)|(&0x3000000<<4)|(&0xC0<<6)|(&0xC00000<<8)|(&0xC00000030<<10)|(&0x300000000<<14)|(&0xC000000<<16)|(&0xC0000000000<<48)|(&0xC00000000000<<50)|(&0x30000000C000<<54)|(&0xC0000000<<56)|(&0x3000<<58)|(&0x30000000<<60)|(&0xC000C<<62)],
            172 => [(&0xFFFC03F000000F00<<0)|(&0xF0000000<<4)|(&0xF0<<8)|(&0xC00000F<<16)|(&0x3C00000<<22)|(&0x300000<<28)|(&0x3000000000000<<36)|(&0xF00000000000<<42)|(&0xC00000F0000<<48)|(&0xF000<<56)|(&0xF00000000<<60)],
            174 => [(&0xFFFFFFF0000FFF00<<0)|(&0xC0C0000C<<2)|(&0x30300003<<6)|(&0xC0C0000C0<<58)|(&0x303000030<<62)],
            175 => [(&0xFFFFFFFFF00FFF00<<0)|(&0xC0000C<<2)|(&0x300003<<6)|(&0xC0000C0<<58)|(&0x3000030<<62)],
            180 => [(&0xFFFC03F000000F00<<0)|(&0xF<<4)|(&0x300000<<6)|(&0x3C00000<<10)|(&0xF000000F0<<12)|(&0xC000000<<22)|(&0xC0000000000<<42)|(&0xF00000000000<<48)|(&0xF000<<52)|(&0x30000F0000000<<58)|(&0xF0000<<60)],
            182 => [(&0xFFFFFFF0000FFF00<<0)|(&0x300300030<<2)|(&0x30C00003<<4)|(&0xC030000C0<<60)|(&0xCC00000C<<62)],
            183 => [(&0xFFFFFFFFF00FFF00<<0)|(&0x300030<<2)|(&0xC00003<<4)|(&0x30000C0<<60)|(&0xC00000C<<62)],
        ]);
        let bits = apply_permutation_lut!(u64, bits, twist, [
            4 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0xC000000000000<<1)|(&0x20000000000000<<3)|(&0x110000000000000<<62)|(&0x40000000000000<<63)],
            6 => [(&0xFE07FFFFFFFFFFFF<<0)|(&0x88000000000000<<1)|(&0x10000000000000<<2)|(&0x20000000000000<<62)|(&0x140000000000000<<63)],
            12 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x4000000000000<<1)|(&0x8000000000000<<2)|(&0x10000000000000<<4)|(&0x20000000000000<<61)|(&0x140000000000000<<62)],
            20 => [(&0xFECBFFFFFFFFFFFF<<0)|(&0x24000000000000<<3)|(&0x100000000000000<<60)|(&0x10000000000000<<62)],
            28 => [(&0xFF87FFFFFFFFFFFF<<0)|(&0x8000000000000<<1)|(&0x10000000000000<<2)|(&0x20000000000000<<62)|(&0x40000000000000<<63)],
            36 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x14000000000000<<2)|(&0x8000000000000<<5)|(&0x40000000000000<<60)|(&0x100000000000000<<61)|(&0x20000000000000<<62)],
            44 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x20000000000000<<1)|(&0x4000000000000<<3)|(&0x8000000000000<<5)|(&0x140000000000000<<60)|(&0x10000000000000<<63)],
            52 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x20000000000000<<1)|(&0x44000000000000<<2)|(&0x100000000000000<<61)|(&0x18000000000000<<63)],
            60 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x4000000000000<<2)|(&0x28000000000000<<3)|(&0x140000000000000<<61)|(&0x10000000000000<<62)],
            68 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x28000000000000<<1)|(&0x4000000000000<<6)|(&0x100000000000000<<58)|(&0x50000000000000<<63)],
            70 => [(&0xFF87FFFFFFFFFFFF<<0)|(&0x20000000000000<<1)|(&0x8000000000000<<2)|(&0x40000000000000<<62)|(&0x10000000000000<<63)],
            76 => [(&0xFEB3FFFFFFFFFFFF<<0)|(&0x4000000000000<<1)|(&0x8000000000000<<5)|(&0x40000000000000<<60)|(&0x100000000000000<<62)],
            84 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x8000000000000<<1)|(&0x14000000000000<<4)|(&0x100000000000000<<59)|(&0x20000000000000<<61)|(&0x40000000000000<<63)],
            92 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x8000000000000<<2)|(&0x20000000000000<<3)|(&0x4000000000000<<4)|(&0x100000000000000<<59)|(&0x50000000000000<<62)],
            100 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0xC000000000000<<3)|(&0x10000000000000<<4)|(&0x100000000000000<<60)|(&0x60000000000000<<61)],
            102 => [(&0xFE07FFFFFFFFFFFF<<0)|(&0xA0000000000000<<1)|(&0x8000000000000<<2)|(&0x40000000000000<<62)|(&0x110000000000000<<63)],
            103 => [(&0xFFFFFFFFFFFFFFFF<<0)],
            108 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x18000000000000<<2)|(&0x4000000000000<<6)|(&0x100000000000000<<58)|(&0x60000000000000<<62)],
            116 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x10000000000000<<1)|(&0x4000000000000<<4)|(&0x8000000000000<<5)|(&0x100000000000000<<59)|(&0x40000000000000<<60)|(&0x20000000000000<<63)],
            118 => [(&0xFE7FFFFFFFFFFFFF<<0)|(&0x80000000000000<<1)|(&0x100000000000000<<63)],
            124 => [(&0xFECBFFFFFFFFFFFF<<0)|(&0x10000000000000<<1)|(&0x4000000000000<<6)|(&0x100000000000000<<58)|(&0x20000000000000<<63)],
            132 => [(&0xFF87FFFFFFFFFFFF<<0)|(&0x10000000000000<<1)|(&0x8000000000000<<3)|(&0x40000000000000<<61)|(&0x20000000000000<<63)],
            140 => [(&0xFECBFFFFFFFFFFFF<<0)|(&0x4000000000000<<2)|(&0x10000000000000<<4)|(&0x120000000000000<<61)],
            148 => [(&0xFF87FFFFFFFFFFFF<<0)|(&0x20000000000000<<1)|(&0x8000000000000<<2)|(&0x40000000000000<<62)|(&0x10000000000000<<63)],
            150 => [(&0xFE07FFFFFFFFFFFF<<0)|(&0x90000000000000<<1)|(&0x8000000000000<<3)|(&0x40000000000000<<61)|(&0x120000000000000<<63)],
            156 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x50000000000000<<2)|(&0x4000000000000<<3)|(&0x100000000000000<<60)|(&0x20000000000000<<62)|(&0x8000000000000<<63)],
            164 => [(&0xFE83FFFFFFFFFFFF<<0)|(&0x14000000000000<<1)|(&0x40000000000000<<2)|(&0x100000000000000<<62)|(&0x28000000000000<<63)],
            172 => [(&0xFEB3FFFFFFFFFFFF<<0)|(&0x8000000000000<<3)|(&0x4000000000000<<6)|(&0x100000000000000<<58)|(&0x40000000000000<<61)],
            174 => [(&0xFF87FFFFFFFFFFFF<<0)|(&0x10000000000000<<1)|(&0x8000000000000<<3)|(&0x40000000000000<<61)|(&0x20000000000000<<63)],
            175 => [(&0xFFFFFFFFFFFFFFFF<<0)],
            180 => [(&0xFEB3FFFFFFFFFFFF<<0)|(&0x40000000000000<<2)|(&0x4000000000000<<4)|(&0x100000000000000<<59)|(&0x8000000000000<<63)],
            182 => [(&0xFF87FFFFFFFFFFFF<<0)|(&0x8000000000000<<1)|(&0x10000000000000<<2)|(&0x20000000000000<<62)|(&0x40000000000000<<63)],
            183 => [(&0xFFFFFFFFFFFFFFFF<<0)],
        ]);
        Self { bits }
    }
}
