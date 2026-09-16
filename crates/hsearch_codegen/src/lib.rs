use std::fmt::Write;
use std::path::Path;

use hsearch_core::Axis::{W, X, Y, Z};
use hsearch_core::Facet::{D, F, I, L, O, R, U};
use hsearch_core::{Group, IDENT, Mat4, PieceType, TransformByMat4, Twist, TwistData, Vec4};
use itertools::Itertools;

mod orientation_lut;
mod permutation_lut;
mod util;

use orientation_lut::OrientationLut;
use permutation_lut::PermutationLut;
use textwrap::dedent;
use util::{collect_bits, corners, edges, preserved_bits, ridges};

pub fn generate_all(out_dir: &Path) -> std::io::Result<()> {
    std::fs::write(out_dir.join("stage1.rs"), stage1())?;
    std::fs::write(out_dir.join("stage2.rs"), stage2())?;
    std::fs::write(out_dir.join("stage3.rs"), stage3())?;
    std::fs::write(out_dir.join("stage4.rs"), stage4())?;
    std::fs::write(out_dir.join("v2_stage1.rs"), v2())?;
    std::fs::write(out_dir.join("xy_rot.rs"), xy_rot())?;
    Ok(())
}

fn stage1() -> String {
    use hsearch_core::stage_utils::s1_ro;

    let solved_e: u32 = collect_bits(edges().map(|v| v[X] == 0));
    let solved_r: u64 = collect_bits(ridges().flat_map(|v| [true, v[X] == 0]));

    let is_in_target = |v: Vec4| v[X] == 0 && v[W] >= 0;
    let target_e: u32 = collect_bits(edges().map(is_in_target));
    let target_r: u64 = collect_bits(ridges().map(is_in_target).flat_map(|b| [true, b]));

    let e = PermutationLut::new(edges()).to_rust_code(32, 0, 1, "e");
    let ro = OrientationLut::new(ridges(), 4, s1_ro).to_rust_code(64, 0, 2, "r");
    let rp = PermutationLut::new(ridges()).to_rust_code(64, 0, 2, "r");

    dedent(&format!(
        "
        impl Stage1 {{
            pub const SOLVED: Self = Self {{ e: 0x{solved_e:08x}, r: 0x{solved_r:012x} }};
            pub const TARGET: Self = Self {{ e: 0x{target_e:08x}, r: 0x{target_r:012x} }};

            fn generated_do_twist(self, twist: Twist) -> Self {{
                let Self {{ e, r }} = self;
                let e = {e};
                let r = {ro};
                let r = {rp};
                Self {{ e, r }}
            }}
        }}
        "
    ))
}

fn stage2() -> String {
    let is_in_stage2 = |v: &Vec4| v[X] != 0 || v[W] < 0;

    let s2_ridges = || ridges().filter(is_in_stage2);
    let s2_edge_stickers = || PieceType::Edge.all_stickers().filter(is_in_stage2);
    let s2_corner_stickers = || PieceType::Corner.all_stickers().filter(is_in_stage2);

    assert_eq!(16, s2_ridges().count());
    assert_eq!(28 * 3, s2_edge_stickers().count());
    assert_eq!(16 * 4, s2_corner_stickers().count());

    let solved_r: u16 = collect_bits(s2_ridges().map(|v| v[X] != 0));
    let solved_e: u128 = collect_bits(s2_edge_stickers().map(|v| v[X].abs() == 2));
    let solved_c: u64 = collect_bits(s2_corner_stickers().map(|v| v[X].abs() == 2));
    let solved_re = solved_r as u128 | (solved_e << 16);

    let target_r: u16 = collect_bits(s2_ridges().map(|v| v[X] < 0 && v[W] >= 0));
    let target_e: u128 = collect_bits(s2_edge_stickers().map(|v| v[X] == -2 && v[W] >= 0));
    let target_c: u64 = collect_bits(s2_corner_stickers().map(|v| v[X] == -2 && v[W] >= 0));
    let target_re = target_r as u128 | (target_e << 16);

    // let target_blocks: [(&Group, fn(Vec4) -> bool); 3] = [
    //     (&full, |v| v[X] <= 0 && v[Y] <= 0 && v[Z] <= 0 && v[W] < 0),
    //     (&full, |v| v[Y] <= 0 && v[Z] <= 0 && v[W] < 0),
    //     (&w_sym, |v| v[Z] <= 0 && v[W] < 0),
    // ];

    // let indent = "            ";
    // let target_constants = target_blocks
    //     .into_iter()
    //     .enumerate()
    //     .map(|(i, (symmetry, predicate))| {
    //         let index = i + 1;
    //         let elems = symmetry
    //             .elems()
    //             .into_iter()
    //             .map(|m| {
    //                 let r = collect_bits::<u16>(s2_ridges().map(|v| predicate(m * v)));
    //                 let e = collect_bits::<u64>(s2_edges().flat_map(|v| [predicate(m * v); 2]));
    //                 let c = collect_bits::<u32>(s2_corners().flat_map(|v| [predicate(m * v); 2]));
    //                 (r, e, c)
    //             })
    //             .sorted()
    //             .dedup()
    //             .map(|(r, e, c)| {
    //                 format!("{indent}    Self::new(0x{r:04x}, 0x{e:016x}, 0x{c:08x}),")
    //             })
    //             .join("\n");
    //         format!("pub const TARGET{index}: &[Self] = &[\n{elems}\n{indent}];")
    //     })
    //     .join(&format!("\n{indent}"));

    let re_lut = PermutationLut::new(std::iter::chain(s2_ridges(), s2_edge_stickers()));
    let re = re_lut.to_rust_code(128, 0, 1, "re");
    let c = PermutationLut::new(s2_corner_stickers()).to_rust_code(64, 0, 1, "c");

    let twists = re_lut.allowed_twists();
    let twists_len = twists.len();

    dedent(&format!(
        "
        impl Stage2 {{
            pub const SOLVED: Self = Self {{ re: 0x{solved_re:025x}, c: 0x{solved_c:016x} }};
            pub const TARGET: Self = Self {{ re: 0x{target_re:025x}, c: 0x{target_c:016x} }};
            pub const TWISTS: [Twist; {twists_len}] = {twists:?};

            fn generated_do_twist(self, twist: Twist) -> Self {{
                let Self {{ re, c }} = self;
                let re = {re};
                let c = {c};
                Self {{ re, c }}
            }}
        }}
        ",
    ))
}

fn stage3() -> String {
    use hsearch_core::stage_utils::s3_eo;

    let is_in_stage3 = |v: &Vec4| v[X] == 1 || v[W] == -1;

    let s3_ridges = || ridges().filter(is_in_stage3);
    let s3_edges = || edges().filter(is_in_stage3);
    let s3_corners = || corners().filter(is_in_stage3);

    let solved_r: u16 = collect_bits(s3_ridges().map(|v| v[X] != 0));
    let solved_ec: u64 = collect_bits(std::iter::chain(
        s3_edges().flat_map(|v| [v[X] != 0; 2]),
        s3_corners().flat_map(|_| [false; 2]),
    ));

    assert_eq!(11, s3_ridges().count());
    assert_eq!(20, s3_edges().count());
    assert_eq!(12, s3_corners().count());

    let r_m: u16 = collect_bits(s3_ridges().map(|v| v[X] == 0));
    let e_m: u64 = collect_bits(s3_edges().flat_map(|v| [v[X] == 0, false]));
    let e_rl: u64 = collect_bits(s3_edges().flat_map(|v| [v[X] != 0, false]));
    let c_rl: u64 = collect_bits(s3_corners().flat_map(|v| [v[X] != 0, false]));

    let r = PermutationLut::new(s3_ridges()).to_rust_code(16, 0, 1, "r");
    let eco = OrientationLut::new(s3_edges().chain(s3_corners()), 4, |r, v, o| {
        if v.taxicab_norm() == 3 {
            s3_eo(r, v, o) // edge
        } else {
            hsearch_core::Axis::from_u8(o).transform_by(r) as u8 // corner
        }
    })
    .to_rust_code(64, 0, 2, "ec");
    let ecp_lut = PermutationLut::new(s3_edges().chain(s3_corners()));
    let ecp = ecp_lut.to_rust_code(64, 0, 2, "ec");
    let twists: Vec<Twist> = ecp_lut
        .allowed_twists()
        .into_iter()
        .filter(|t| matches!(t.facet(), R | I) || X.transform_by(t.data().rot) == X)
        .collect();

    dedent(&format!(
        "
        impl Stage3 {{
            pub const SOLVED: Self = Self {{ r: 0x{solved_r:04x}, ec: 0x{solved_ec:016x} }};
            pub const TWISTS: [Twist; {twists_len}] = {twists:?};

            /// `M`-slice ridges.
            const R_M: u16 = 0x{r_m:04x};
            /// `M`-slice edges (low bits only).
            const E_M: u64 = 0x{e_m:010x};
            /// `R`/`L` edges (low bits only).
            const E_RL: u64 = 0x{e_rl:010x};
            /// Corners (low bits only). All corners are in `R`/`L`
            const C_RL: u64 = 0x{c_rl:06x} << 40;

            fn generated_do_twist(self, twist: Twist) -> Self {{
                let Self {{ r, ec }} = self;
                let r = {r};
                let ec = {eco};
                let ec = {ecp};
                Self {{ r, ec }}
            }}
        }}
        ",
        twists_len = twists.len(),
    ))
}

fn stage4_corners() -> impl Iterator<Item = Vec4> {
    corners().filter(|&v| v[Z] == 1 || (v[W] == 1 && v[Y] == 1))
}
fn stage4_edges() -> impl Iterator<Item = Vec4> {
    edges().filter(|&v| v[Z] == 1 || (v[W] == 1 && v[Y] == 1))
}
fn stage4_ridges() -> impl Iterator<Item = Vec4> {
    ridges().filter(|&v| v[Z] == 1 || (v[W] == 1 && v[Y] == 1))
}

fn stage4_rotation(t: TwistData) -> Result<Option<Mat4>, ()> {
    match t.facet {
        F => Ok(None),
        I if F.transform_by(t.rot) == F => Ok(None),
        O => {
            let new_f = F.transform_by(t.rot);
            let new_u = U.transform_by(t.rot);
            if new_f == F {
                Ok(Some(t.rot.inv()))
            } else if new_u == F {
                match new_f {
                    U => Ok(None),
                    R | L => Ok(Some(new_f.mat4_to(U))),
                    D => Ok(Some(Mat4::rot(X, Y).pow(2))),
                    _ => Err(()),
                }
            } else {
                Err(())
            }
        }
        _ => Err(()),
    }
}

fn stage4() -> String {
    let solved: u64 = collect_bits(
        stage4_corners()
            .flat_map(|_| [true, true])
            .chain(stage4_edges().flat_map(|v| [v[W] != 0, false]))
            .chain(stage4_ridges().map(|v| v[W] != 0)),
    );
    let mut by_pow = [Vec::new(), Vec::new(), Vec::new()];
    for t in Twist::iter() {
        if let Ok(Some(m)) = stage4_rotation(t.data()) {
            let base = Mat4::rot(X, Y);
            let rot = (1..=3).find(|&p| base.pow(p) == m).unwrap();
            by_pow[rot as usize - 1].push(t);
        }
    }
    let mut match_src = String::from(
        "    fn generated_implicit_rotation_after_twist(twist: Twist) -> XyRot {
        match twist.to_index() {
",
    );
    for (i, ts) in by_pow.iter().enumerate() {
        if !ts.is_empty() {
            write!(
                match_src,
                "            {} => XyRot::from_pow({}),
",
                ts.iter().map(|t| t.to_index()).join(" | "),
                i + 1
            )
            .unwrap();
        }
    }
    match_src.push_str(
        "            _ => XyRot::IDENT,
        }
    }
",
    );
    let o = OrientationLut::with_action(stage4_corners().chain(stage4_edges()), 4, |t, v, old| {
        let mut r = if t.affects(v) { t.rot } else { IDENT };
        if let Some(extra) = stage4_rotation(t).ok()? {
            r = extra * r;
        }
        Some(if v.taxicab_norm() == 3 {
            match old {
                0 => old,
                _ => {
                    let old_axis = v.nonzero_axes().nth(3 - old as usize).unwrap();
                    let new_axis = old_axis.transform_by(r);
                    3 - (r * v).nonzero_axes().position(|a| a == new_axis).unwrap() as u8
                }
            }
        } else {
            hsearch_core::Axis::from_u8(old).transform_by(r) as u8
        })
    })
    .to_rust_code(64, 0, 2, "bits");
    let p = PermutationLut::with_action(stage4_corners().chain(stage4_edges()), |t, v| {
        let p = if t.affects(v) { t.rot * v } else { v };
        let extra = stage4_rotation(t).ok()?;
        Some(if let Some(extra) = extra {
            extra * p
        } else {
            p
        })
    });
    let p2 = p.to_rust_code(64, 0, 2, "bits");
    let r = PermutationLut::with_action(stage4_ridges(), |t, v| {
        stage4_rotation(t)
            .ok()
            .map(|extra| extra.unwrap_or(IDENT) * if t.affects(v) { t.rot * v } else { v })
    })
    .to_rust_code(64, 50, 1, "bits");
    let twists = p.allowed_twists();
    let mut out = String::new();
    write!(
        out,
        "impl Stage4 {{
    pub const SOLVED: Self = Self {{ bits: 0x{solved:016x} }};
}}

impl Stage4 {{
{match_src}    fn generated_do_twist(self, twist: Twist) -> Self {{
        let Self {{ bits }} = self;
        let bits = {o};
        let bits = {p2};
        let bits = {r};
        Self {{ bits }}
    }}
}}

impl Stage4 {{ pub const TWISTS: [Twist; {}] = {:?}; }}
",
        twists.len(),
        twists,
    )
    .unwrap();
    out
}

fn v2_stickers(ty: PieceType) -> impl Iterator<Item = Vec4> {
    hsearch_core::Facet::ALL
        .iter()
        .flat_map(move |&f| {
            let mut min = Vec4([-1; 4]);
            let mut max = Vec4([1; 4]);
            min[f.axis()] = f.sign() as i8 * 2;
            max[f.axis()] = f.sign() as i8 * 2;
            Vec4::region(min, max)
        })
        .filter(move |v| v.taxicab_norm() - 1 == ty.sticker_count())
}

fn v2() -> String {
    let solved_r =
        collect_bits::<u64>(v2_stickers(PieceType::Ridge).map(|v| v.unwrap_sticker().1 == W));
    let solved_e =
        collect_bits::<u128>(v2_stickers(PieceType::Edge).map(|v| v.unwrap_sticker().1 == W));
    let solved_c =
        collect_bits::<u64>(v2_stickers(PieceType::Corner).map(|v| v.unwrap_sticker().1 == W));
    let symmetry = Group::new(vec![
        Mat4::refl(X),
        Mat4::rot(X, Y),
        Mat4::rot(X, Z),
        Mat4::refl(W),
    ])
    .elems();
    let blocks: [fn(Vec4) -> bool; 3] = [
        |v| v[X] <= 0 && v[Y] <= 0 && v[Z] <= 0 && v[W] == -2,
        |v| v[Y] <= 0 && v[Z] <= 0 && v[W] == -2,
        |v| v[Z] <= 0 && v[W] == -2,
    ];
    let mut targets = String::new();
    for (i, pred) in blocks.into_iter().enumerate() {
        write!(
            targets,
            "    pub const TARGET{}: &[Self] = &[
",
            i + 1
        )
        .unwrap();
        for (r, e, c) in symmetry
            .iter()
            .map(|&m| {
                (
                    collect_bits::<u64>(v2_stickers(PieceType::Ridge).map(|v| pred(m * v))),
                    collect_bits::<u128>(v2_stickers(PieceType::Edge).map(|v| pred(m * v))),
                    collect_bits::<u64>(v2_stickers(PieceType::Corner).map(|v| pred(m * v))),
                )
            })
            .sorted()
            .dedup()
        {
            write!(
                targets,
                "        V2Stage1 {{ r: 0x{r:012x}, e: 0x{e:024x}, c: 0x{c:016x} }},
"
            )
            .unwrap();
        }
        targets.push_str(
            "    ];
",
        );
    }
    let r = PermutationLut::new(v2_stickers(PieceType::Ridge)).to_rust_code(64, 0, 1, "r");
    let e = PermutationLut::new(v2_stickers(PieceType::Edge)).to_rust_code(128, 0, 1, "e");
    let c = PermutationLut::new(v2_stickers(PieceType::Corner)).to_rust_code(64, 0, 1, "c");
    let mut out = String::new();
    write!(
        out,
        "impl V2Stage1 {{
    pub const SOLVED: Self = V2Stage1 {{ r: 0x{solved_r:012x}, e: 0x{solved_e:024x}, c: 0x{solved_c:016x} }};
{targets}}}

impl V2Stage1 {{
    fn generated_do_twist(self, twist: Twist) -> Self {{
        let Self {{ r, e, c }} = self;
        let r = {r};
        let e = {e};
        let c = {c};
        Self {{ r, e, c }}
    }}
}}
"
    )
    .unwrap();
    out
}

fn xy_rot() -> String {
    let mut rows = String::new();
    for pow in [1, 2, 3] {
        let m = Mat4::rot(X, Y).pow(pow);
        write!(
            rows,
            "    {:?},
",
            Twist::iter().map(|t| t.transform_by(m).0).collect_vec()
        )
        .unwrap();
    }
    let mut out = String::new();
    write!(
        out,
        "const MUL_XY_ROT_TWIST: [[u8; {}]; 3] = [
{rows}];
",
        Twist::iter().len(),
    )
    .unwrap();
    out
}
