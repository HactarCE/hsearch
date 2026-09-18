use std::path::Path;

use hsearch_core::*;

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
    Ok(())
}

fn stage1() -> String {
    use hsearch_core::stage_utils::xyz_ro;

    let solved_e: u32 = collect_bits(edges().map(|v| v[X] == 0));
    let solved_r: u64 = collect_bits(ridges().flat_map(|v| [true, v[X] == 0]));

    let is_in_target = |v: Vec4| v[X] == 0 && v[W] >= 0;
    let target_e: u32 = collect_bits(edges().map(is_in_target));
    let target_r: u64 = collect_bits(ridges().map(is_in_target).flat_map(|b| [true, b]));

    let e = PermutationLut::new(edges()).to_rust_code(32, 0, 1, "e");
    let ro = OrientationLut::new(ridges(), 4, xyz_ro).to_rust_code(64, 0, 2, "r");
    let rp = PermutationLut::new(ridges()).to_rust_code(64, 0, 2, "r");

    let twist_set = TwistSet::ALL;

    dedent(&format!(
        "
        impl Stage1 {{
            pub const SOLVED: Self = Self {{ e: 0x{solved_e:08x}, r: 0x{solved_r:012x} }};
            pub const TARGET: Self = Self {{ e: 0x{target_e:08x}, r: 0x{target_r:012x} }};

            const GENERATED_TWISTS: TwistSet = {twist_set:?};

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

    let twist_set = re_lut.allowed_twists();

    dedent(&format!(
        "
        impl Stage2 {{
            pub const SOLVED: Self = Self {{ re: 0x{solved_re:025x}, c: 0x{solved_c:016x} }};
            pub const TARGET: Self = Self {{ re: 0x{target_re:025x}, c: 0x{target_c:016x} }};

            const GENERATED_TWISTS: TwistSet = {twist_set:?};

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
    use hsearch_core::stage_utils::rl_eo;

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
            rl_eo(r, v, o) // edge
        } else {
            Axis::from_u8(o).transform_by(r) as u8 // corner
        }
    })
    .to_rust_code(64, 0, 2, "ec");
    let ecp_lut = PermutationLut::new(s3_edges().chain(s3_corners()));
    let ecp = ecp_lut.to_rust_code(64, 0, 2, "ec");

    let twist_set = ecp_lut
        .allowed_twists()
        .filter(|t| matches!(t.facet(), R | I) || X.transform_by(t.rot()) == X);

    dedent(&format!(
        "
        impl Stage3 {{
            pub const SOLVED: Self = Self {{ r: 0x{solved_r:04x}, ec: 0x{solved_ec:016x} }};

            /// `M`-slice ridges.
            const R_M: u16 = 0x{r_m:04x};
            /// `M`-slice edges (low bits only).
            const E_M: u64 = 0x{e_m:010x};
            /// `R`/`L` edges (low bits only).
            const E_RL: u64 = 0x{e_rl:010x};
            /// Corners (low bits only). All corners are in `R`/`L`
            const C_RL: u64 = 0x{c_rl:06x} << 40;

            const GENERATED_TWISTS: TwistSet = {twist_set:?};

            fn generated_do_twist(self, twist: Twist) -> Self {{
                let Self {{ r, ec }} = self;
                let r = {r};
                let ec = {eco};
                let ec = {ecp};
                Self {{ r, ec }}
            }}
        }}
        "
    ))
}

fn stage4() -> String {
    use hsearch_core::stage_utils::rl_eo;

    let solved_r: &str = "[RidgePos(0); 4]"; // solved state is not representible
    let solved_e: u64 = collect_bits(edges().flat_map(|v| [v[X] != 0; 2]));
    let solved_c: u32 = collect_bits(corners().flat_map(|_| [false; 2]));

    let eo = OrientationLut::new(edges(), 4, rl_eo).to_rust_code(64, 0, 2, "e");
    let ep = PermutationLut::new(edges()).to_rust_code(64, 0, 2, "e");
    let co = OrientationLut::new(corners(), 4, |r, _v, o| {
        Axis::from_u8(o).transform_by(r) as u8
    })
    .to_rust_code(32, 0, 2, "c");
    let cp = PermutationLut::new(corners()).to_rust_code(32, 0, 2, "c");

    let twist_set = TwistSet::new(|t| X.transform_by(t.rot()) == X);

    dedent(&format!(
        "
        impl Stage4 {{
            pub const SOLVED: Self = Self {{ r: {solved_r}, e: 0x{solved_e:016x}, c: 0x{solved_c:08x} }};

            const GENERATED_TWISTS: TwistSet = {twist_set:?};

            fn generated_do_twist(self, twist: Twist) -> Self {{
                let Self {{ r, e, c }} = self;
                let r = update_ridges(r, twist);
                let e = {eo};
                let e = {ep};
                let c = {co};
                let c = {cp};
                Self {{ r, e, c }}
            }}
        }}
        "
    ))
}
