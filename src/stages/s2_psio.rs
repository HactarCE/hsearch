use std::ops::BitAnd;

use super::*;

/// Stage 2: P separation + I/O edge & corner orientation
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Stage2 {
    /// For each ridge, 1 bit indicating one of the following cases:
    ///
    /// - `0` = belongs in the P slice
    /// - `1` = belongs in I/O
    pub r_p: u16, // u16

    /// For each edge, 2 bits indicating one of the following cases:
    ///
    /// - `00` = belongs in P slice, any orientation
    /// - `01` = belongs in I/O, good orientation
    /// - `10` = belongs in I/O, bad orientation 1
    /// - `11` = belongs in I/O, bad orientation 2
    pub e_op: u64, // u56

    /// For each corner, 2 bits indicating the axis containing its I/O sticker:
    ///
    /// - `00` = X
    /// - `01` = Y
    /// - `10` = Z
    /// - `11` = W
    pub c_o: u32, // u32
}

impl Default for Stage2 {
    fn default() -> Self {
        Self::SOLVED
    }
}

impl BitAnd for Stage2 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            r_p: self.r_p & rhs.r_p,
            e_op: self.e_op & rhs.e_op,
            c_o: self.c_o & rhs.c_o,
        }
    }
}

impl Stage2 {
    pub const SOLVED: Self = Self {
        r_p: 0xfc3f,
        e_op: 0x0055555500555555,
        c_o: 0xffffffff,
    };

    pub const TARGET1: &[Self] = &[
        Self::new(0x0007, 0x000000000000030f, 0x00000003),
        Self::new(0x000b, 0x0000000000000c33, 0x0000000c),
        Self::new(0x0015, 0x00000000000030cc, 0x00000030),
        Self::new(0x0019, 0x000000000000c0f0, 0x000000c0),
        Self::new(0x0026, 0x00000000000f0300, 0x00000300),
        Self::new(0x002a, 0x0000000000330c00, 0x00000c00),
        Self::new(0x0034, 0x0000000000cc3000, 0x00003000),
        Self::new(0x0038, 0x0000000000f0c000, 0x0000c000),
        Self::new(0x1c00, 0x0000030f00000000, 0x00030000),
        Self::new(0x2c00, 0x00000c3300000000, 0x000c0000),
        Self::new(0x5400, 0x000030cc00000000, 0x00300000),
        Self::new(0x6400, 0x0000c0f000000000, 0x00c00000),
        Self::new(0x9800, 0x000f030000000000, 0x03000000),
        Self::new(0xa800, 0x00330c0000000000, 0x0c000000),
        Self::new(0xd000, 0x00cc300000000000, 0x30000000),
        Self::new(0xe000, 0x00f0c00000000000, 0xc0000000),
    ];
    pub const TARGET2: &[Self] = &[
        Self::new(0x000f, 0x0000000000000f3f, 0x0000000f),
        Self::new(0x001d, 0x000000000000f0fc, 0x000000f0),
        Self::new(0x002e, 0x00000000003f0f00, 0x00000f00),
        Self::new(0x003c, 0x0000000000fcf000, 0x0000f000),
        Self::new(0x3c00, 0x00000f3f00000000, 0x000f0000),
        Self::new(0x7400, 0x0000f0fc00000000, 0x00f00000),
        Self::new(0xb800, 0x003f0f0000000000, 0x0f000000),
        Self::new(0xf000, 0x00fcf00000000000, 0xf0000000),
    ];
    pub const TARGET3: &[Self] = &[
        Self::new(0x001f, 0x000000000000ffff, 0x000000ff),
        Self::new(0x003e, 0x0000000000ffff00, 0x0000ff00),
        Self::new(0x7c00, 0x0000ffff00000000, 0x00ff0000),
        Self::new(0xf800, 0x00ffff0000000000, 0xff000000),
    ];

    const fn new(r_p: u16, e_op: u64, c_o: u32) -> Self {
        Self { r_p, e_op, c_o }
    }

    pub fn is_target_solved(self, target: &[Self]) -> bool {
        target.iter().any(|&t| self & t == Self::SOLVED & t)
    }

    pub fn good_ridges(self) -> u8 {
        (self.r_p & Self::SOLVED.r_p).count_ones() as u8
    }
    pub fn good_edges(self) -> u8 {
        (self.e_op & !(self.e_op >> 1) & Self::SOLVED.e_op).count_ones() as u8
    }
    pub fn good_corners(self) -> u8 {
        (self.c_o & self.c_o >> 1 & 0x5555_5555).count_ones() as u8
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    use crate::lut_gen::*;

    fn ridges() -> impl Iterator<Item = Vec4> {
        PieceType::Ridge.iter().filter(|v| v[W] != 0 || v[Z] == 1)
    }
    fn edges() -> impl Iterator<Item = Vec4> {
        PieceType::Edge.iter().filter(|v| v[W] != 0 || v[Z] == 1)
    }
    fn corners() -> impl Iterator<Item = Vec4> {
        PieceType::Corner.iter()
    }

    #[test]
    fn print_stage2_constants() {
        println!();

        println!("pub const SOLVED: Self = Self {{");
        let m = crate::util::collect_bits(ridges().map(|v| v[W] != 0));
        println!("    r_p: 0x{m:04x},");
        let m = crate::util::collect_bits(edges().flat_map(|v| [v[W] != 0, false]));
        println!("    e_op: 0x{m:016x},");
        let m = crate::util::collect_bits(corners().flat_map(|_| [true, true]));
        println!("    c_o: 0x{m:08x},");
        println!("}};");
        println!();

        let target_blocks: [fn(Vec4) -> bool; _] = [
            |v| v[X] <= 0 && v[Y] <= 0 && v[Z] <= 0 && v[W] < 0, // 2x2x2x1
            |v| v[Y] <= 0 && v[Z] <= 0 && v[W] < 0,              // 3x2x2x1
            |v| v[Z] <= 0 && v[W] < 0,                           // 3x3x2x1
        ];
        let symmetry = crate::group::Group::new(Axis::ALL.map(Mat4::refl).to_vec());
        for (i, block_predicate) in target_blocks.into_iter().enumerate() {
            println!("pub const TARGET{}: &[Self] = &[", i + 1);
            symmetry
                .elems()
                .into_iter()
                .map(|m| {
                    use crate::util::collect_bits;
                    let r_p = collect_bits(ridges().map(|v| block_predicate(m * v)));
                    let e_op = collect_bits(edges().flat_map(|v| [block_predicate(m * v); 2]));
                    let c_o = collect_bits(corners().flat_map(|v| [block_predicate(m * v); 2]));
                    (r_p, e_op, c_o)
                })
                .sorted()
                .dedup()
                .for_each(|(r_p, e_op, c_o)| {
                    println!("    Self::new(0x{r_p:04x}, 0x{e_op:016x}, 0x{c_o:08x}),");
                });
            println!("];");
        }
    }

    #[test]
    fn lutgen_stage2() {
        println!();
        println!("let Self {{ r_p, e_op, c_o }} = self;");

        let lut1 = PermutationLut::new(ridges());
        println!("let r_p = {};", lut1.to_rust_code(16, 0, 1, "r_p"));

        let lut2 = OrientationLut::new(edges(), 4, |r, v, o| {
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
        });
        println!("let e_op = {};", lut2.to_rust_code(64, 0, 2, "e_op"));

        let lut3 = PermutationLut::new(edges());
        println!("let e_op = {};", lut3.to_rust_code(64, 0, 2, "e_op"));

        let lut4 = OrientationLut::new(corners(), 4, |r, _v, o| {
            Axis::from_u8(o).transform_by(r) as u8
        });
        println!("let c_o = {};", lut4.to_rust_code(32, 0, 2, "c_o"));

        let lut5 = PermutationLut::new(corners());
        println!("let c_o = {};", lut5.to_rust_code(32, 0, 2, "c_o"));

        println!("Self {{ r_p, e_op, c_o }}");
        println!();

        println!();
        let twists = lut1.allowed_twists();
        println!(
            "pub const TWISTS: [Twist; {}] = {:?};",
            twists.len(),
            twists
        );
        println!();
    }
}

impl Stage2 {
    pub const TWISTS: [Twist; 80] = [
        Twist(4),
        Twist(6),
        Twist(7),
        Twist(12),
        Twist(14),
        Twist(15),
        Twist(20),
        Twist(22),
        Twist(23),
        Twist(28),
        Twist(30),
        Twist(31),
        Twist(36),
        Twist(38),
        Twist(39),
        Twist(44),
        Twist(46),
        Twist(47),
        Twist(52),
        Twist(54),
        Twist(55),
        Twist(60),
        Twist(61),
        Twist(62),
        Twist(63),
        Twist(68),
        Twist(69),
        Twist(70),
        Twist(71),
        Twist(76),
        Twist(77),
        Twist(78),
        Twist(79),
        Twist(84),
        Twist(85),
        Twist(86),
        Twist(87),
        Twist(92),
        Twist(94),
        Twist(95),
        Twist(100),
        Twist(102),
        Twist(103),
        Twist(108),
        Twist(109),
        Twist(110),
        Twist(111),
        Twist(116),
        Twist(117),
        Twist(118),
        Twist(119),
        Twist(124),
        Twist(126),
        Twist(127),
        Twist(132),
        Twist(134),
        Twist(135),
        Twist(140),
        Twist(142),
        Twist(143),
        Twist(148),
        Twist(150),
        Twist(151),
        Twist(156),
        Twist(158),
        Twist(159),
        Twist(161),
        Twist(163),
        Twist(164),
        Twist(165),
        Twist(166),
        Twist(167),
        Twist(168),
        Twist(170),
        Twist(172),
        Twist(174),
        Twist(175),
        Twist(180),
        Twist(182),
        Twist(183),
    ];
}

impl Stage for Stage2 {
    fn with_setup(twists: &[Twist]) -> Self {
        let state = twists
            .iter()
            .copied()
            .fold(SimplePuzzleSim::default(), SimplePuzzleSim::do_twist);
        Self {
            r_p: state.to_bits(
                1,
                PieceType::Ridge,
                |v| v[W] != 0 || v[Z] == 1,
                |init, _att| (init[W] != 0) as u64,
            ) as u16,
            e_op: state.to_bits(
                2,
                PieceType::Edge,
                |v| v[W] != 0 || v[Z] == 1,
                |init, att| {
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
                },
            ),
            c_o: state.to_bits(
                2,
                PieceType::Corner,
                |_| true,
                |_init, att| W.transform_by(att) as u64,
            ) as u32,
        }
    }

    fn is_solved(self) -> bool {
        self == Self::default()
    }

    fn do_twist(self, twist: Twist) -> Self {
        let Self { r_p, e_op, c_o } = self;
        let r_p = apply_permutation_lut!(u16, r_p, twist, [
            4 => [(&0x7C1F<<0)|(&0x60<<1)|(&0x100<<7)|(&0x8000<<10)|(&0x80<<14)|(&0x200<<15)],
            6 => [(&0x3FF<<0)|(&0x4000<<1)|(&0x1000<<2)|(&0x400<<3)|(&0x8000<<13)|(&0x2000<<14)|(&0x800<<15)],
            7 => [(&0xFFC0<<0)|(&0x1<<1)|(&0x2<<2)|(&0x4<<3)|(&0x8<<13)|(&0x10<<14)|(&0x20<<15)],
            12 => [(&0x7C1F<<0)|(&0x20<<1)|(&0x40<<2)|(&0x80<<8)|(&0x8000<<10)|(&0x100<<13)|(&0x200<<14)],
            14 => [(&0x3FF<<0)|(&0xC00<<1)|(&0x2000<<2)|(&0x1000<<14)|(&0xC000<<15)],
            15 => [(&0xFFD2<<0)|(&0x8<<2)|(&0x1<<3)|(&0x20<<13)|(&0x4<<14)],
            20 => [(&0x7E5F<<0)|(&0x20<<3)|(&0x100<<7)|(&0x8000<<8)|(&0x80<<14)],
            22 => [(&0x3FF<<0)|(&0x800<<1)|(&0x1000<<3)|(&0x400<<4)|(&0x8000<<12)|(&0x2000<<13)|(&0x4000<<15)],
            23 => [(&0xFFC0<<0)|(&0x8<<1)|(&0x1<<3)|(&0x2<<4)|(&0x10<<12)|(&0x20<<13)|(&0x4<<15)],
            28 => [(&0xFC3F<<0)|(&0x40<<1)|(&0x80<<2)|(&0x100<<14)|(&0x200<<15)],
            30 => [(&0x3FF<<0)|(&0x2800<<2)|(&0x400<<4)|(&0x8000<<12)|(&0x5000<<14)],
            31 => [(&0xFFC0<<0)|(&0x18<<1)|(&0x1<<2)|(&0x20<<14)|(&0x6<<15)],
            36 => [(&0x7C1F<<0)|(&0xA0<<2)|(&0x8040<<9)|(&0x200<<12)|(&0x100<<14)],
            38 => [(&0x3FF<<0)|(&0x1C00<<3)|(&0xE000<<13)],
            39 => [(&0xFFC0<<0)|(&0x5<<2)|(&0x2<<4)|(&0x10<<12)|(&0x28<<14)],
            44 => [(&0x7C1F<<0)|(&0x100<<1)|(&0x20<<3)|(&0x8000<<8)|(&0x40<<9)|(&0x200<<12)|(&0x80<<15)],
            46 => [(&0x3FF<<0)|(&0x2400<<2)|(&0x800<<3)|(&0x4000<<13)|(&0x9000<<14)],
            47 => [(&0xFFC0<<0)|(&0x9<<2)|(&0x2<<3)|(&0x10<<13)|(&0x24<<14)],
            52 => [(&0x7C1F<<0)|(&0x100<<1)|(&0x20<<2)|(&0x200<<6)|(&0x8000<<9)|(&0xC0<<15)],
            54 => [(&0x4BFF<<0)|(&0x1000<<1)|(&0x400<<5)|(&0x8000<<11)|(&0x2000<<15)],
            55 => [(&0xFFCC<<0)|(&0x1<<1)|(&0x2<<4)|(&0x10<<12)|(&0x20<<15)],
            60 => [(&0x7C1F<<0)|(&0x20<<2)|(&0x40<<3)|(&0x100<<7)|(&0x8000<<9)|(&0x200<<13)|(&0x80<<14)],
            61 => [(&0xFBFE<<0)|(&0x400<<6)|(&0x1<<10)],
            62 => [(&0x4BFF<<0)|(&0x400<<2)|(&0x1000<<3)|(&0x2000<<13)|(&0x8000<<14)],
            63 => [(&0xFFC0<<0)|(&0xA<<2)|(&0x1<<4)|(&0x20<<12)|(&0x14<<14)],
            68 => [(&0x7C1F<<0)|(&0x140<<1)|(&0x8000<<6)|(&0x20<<10)|(&0x280<<15)],
            69 => [(&0xFBFE<<0)|(&0x400<<6)|(&0x1<<10)],
            70 => [(&0x87FF<<0)|(&0x800<<1)|(&0x1000<<2)|(&0x2000<<14)|(&0x4000<<15)],
            71 => [(&0xFFC0<<0)|(&0x2<<1)|(&0x4<<3)|(&0x1<<4)|(&0x20<<12)|(&0x8<<13)|(&0x10<<15)],
            76 => [(&0x7D9F<<0)|(&0x20<<1)|(&0x40<<9)|(&0x8000<<10)|(&0x200<<12)],
            77 => [(&0xFBFE<<0)|(&0x400<<6)|(&0x1<<10)],
            78 => [(&0x4BFF<<0)|(&0x2000<<2)|(&0x400<<3)|(&0x8000<<13)|(&0x1000<<14)],
            79 => [(&0xFFC0<<0)|(&0x7<<3)|(&0x38<<13)],
            84 => [(&0x7C1F<<0)|(&0x40<<1)|(&0x20<<4)|(&0x8000<<7)|(&0x80<<8)|(&0x100<<13)|(&0x200<<15)],
            85 => [(&0xFFFF<<0)],
            86 => [(&0x3FF<<0)|(&0x1400<<2)|(&0x800<<4)|(&0x4000<<12)|(&0xA000<<14)],
            87 => [(&0xFFC0<<0)|(&0x4<<1)|(&0x3<<4)|(&0x30<<12)|(&0x8<<15)],
            92 => [(&0x7C1F<<0)|(&0x40<<2)|(&0x20<<4)|(&0x8100<<7)|(&0x280<<14)],
            94 => [(&0x3FF<<0)|(&0x2000<<1)|(&0x400<<3)|(&0x800<<4)|(&0x4000<<12)|(&0x8000<<13)|(&0x1000<<15)],
            95 => [(&0xFFD2<<0)|(&0x4<<1)|(&0x1<<5)|(&0x20<<11)|(&0x8<<15)],
            100 => [(&0x7C1F<<0)|(&0x60<<3)|(&0x8080<<8)|(&0x300<<13)],
            102 => [(&0x3FF<<0)|(&0x6000<<1)|(&0x400<<2)|(&0x8000<<14)|(&0x1800<<15)],
            103 => [(&0xFFE1<<0)|(&0x8<<1)|(&0x2<<2)|(&0x10<<14)|(&0x4<<15)],
            108 => [(&0x7C1F<<0)|(&0xC0<<2)|(&0x8000<<6)|(&0x20<<10)|(&0x300<<14)],
            109 => [(&0xFFFF<<0)],
            110 => [(&0x3FF<<0)|(&0x1000<<1)|(&0xC00<<4)|(&0xC000<<12)|(&0x2000<<15)],
            111 => [(&0xFFD2<<0)|(&0x1<<2)|(&0x4<<3)|(&0x8<<13)|(&0x20<<14)],
            116 => [(&0x7C1F<<0)|(&0x80<<1)|(&0x20<<4)|(&0x8000<<7)|(&0x40<<9)|(&0x200<<12)|(&0x100<<15)],
            117 => [(&0xFFFF<<0)],
            118 => [(&0x3FF<<0)|(&0x5400<<1)|(&0xA800<<15)],
            119 => [(&0xFFC0<<0)|(&0x10<<1)|(&0x4<<2)|(&0x1<<3)|(&0x20<<13)|(&0x8<<14)|(&0x2<<15)],
            124 => [(&0x7E5F<<0)|(&0x80<<1)|(&0x8000<<6)|(&0x20<<10)|(&0x100<<15)],
            126 => [(&0x3FF<<0)|(&0x2800<<1)|(&0x400<<5)|(&0x8000<<11)|(&0x5000<<15)],
            127 => [(&0xFFC0<<0)|(&0x15<<1)|(&0x2A<<15)],
            132 => [(&0xFC3F<<0)|(&0x80<<1)|(&0x40<<3)|(&0x200<<13)|(&0x100<<15)],
            134 => [(&0x33FF<<0)|(&0x800<<3)|(&0x400<<5)|(&0x8000<<11)|(&0x4000<<13)],
            135 => [(&0xFFC0<<0)|(&0x6<<2)|(&0x1<<5)|(&0x20<<11)|(&0x18<<14)],
            140 => [(&0x7E5F<<0)|(&0x20<<2)|(&0x80<<8)|(&0x8000<<9)|(&0x100<<13)],
            142 => [(&0x33FF<<0)|(&0x400<<1)|(&0x800<<4)|(&0x4000<<12)|(&0x8000<<15)],
            143 => [(&0xFFCC<<0)|(&0x2<<3)|(&0x1<<5)|(&0x20<<11)|(&0x10<<13)],
            148 => [(&0xFC3F<<0)|(&0x100<<1)|(&0x40<<2)|(&0x200<<14)|(&0x80<<15)],
            150 => [(&0x33FF<<0)|(&0x4000<<1)|(&0x400<<4)|(&0x8000<<12)|(&0x800<<15)],
            151 => [(&0xFFCC<<0)|(&0x10<<1)|(&0x1<<4)|(&0x20<<12)|(&0x2<<15)],
            156 => [(&0x7C1F<<0)|(&0x80<<2)|(&0x20<<3)|(&0x200<<6)|(&0x8000<<8)|(&0x100<<14)|(&0x40<<15)],
            158 => [(&0x3FF<<0)|(&0x400<<1)|(&0x800<<2)|(&0x1000<<3)|(&0x2000<<13)|(&0x4000<<14)|(&0x8000<<15)],
            159 => [(&0xFFC0<<0)|(&0x3<<1)|(&0x8<<2)|(&0x4<<14)|(&0x30<<15)],
            161 => [(&0xEFFB<<0)|(&0x1000<<6)|(&0x4<<10)],
            163 => [(&0xF7FD<<0)|(&0x800<<6)|(&0x2<<10)],
            164 => [(&0x7C1F<<0)|(&0xA0<<1)|(&0x200<<6)|(&0x8000<<10)|(&0x140<<15)],
            165 => [(&0xFBFE<<0)|(&0x400<<6)|(&0x1<<10)],
            166 => [(&0x3FF<<0)|(&0x1800<<2)|(&0x400<<5)|(&0x8000<<11)|(&0x6000<<14)],
            167 => [(&0xFFC0<<0)|(&0xA<<1)|(&0x1<<5)|(&0x20<<11)|(&0x14<<15)],
            168 => [(&0xDFF7<<0)|(&0x2000<<6)|(&0x8<<10)],
            170 => [(&0xBFEF<<0)|(&0x4000<<6)|(&0x10<<10)],
            172 => [(&0x7D9F<<0)|(&0x40<<3)|(&0x8000<<6)|(&0x20<<10)|(&0x200<<13)],
            174 => [(&0x87FF<<0)|(&0x1000<<1)|(&0x800<<3)|(&0x4000<<13)|(&0x2000<<15)],
            175 => [(&0xFFE1<<0)|(&0x4<<1)|(&0x2<<3)|(&0x10<<13)|(&0x8<<15)],
            180 => [(&0x7D9F<<0)|(&0x20<<4)|(&0x200<<6)|(&0x8000<<7)|(&0x40<<15)],
            182 => [(&0x87FF<<0)|(&0x2000<<1)|(&0x800<<2)|(&0x4000<<14)|(&0x1000<<15)],
            183 => [(&0xFFE1<<0)|(&0x2<<1)|(&0x4<<2)|(&0x8<<14)|(&0x10<<15)],
        ]);
        let e_op = crate::lut::update_orientations_u64(
            e_op,
            [
                [0x0, 0xFF33FF33FF33FF, 0xCC00CC00CC00],
                [0x0, 0xFFCCFFCCFFCCFF, 0x330033003300],
                [0x0, 0x7FFF7FFF7FFF7F, 0xC0A0C050C0A0C0],
                [0x0, 0xFFFAFFFFFFFAFF, 0x10F010A010F01],
                [0x0, 0x3CFFFFAA3CFFFF, 0xEB0000FFEB0000],
                [0x0, 0xFFFFD7FFFFFFD7, 0xBE000000BE],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55005500000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x145514],
                [0x0, 0xCFFFCFFFCFFFCF, 0x30003000300030],
                [0x0, 0xF3FFF3FFF3FFF3, 0xC000C000C000C],
                [0x0, 0x7FFF7FFF7FFF7F, 0xC0A0C050C0A0C0],
                [0x0, 0xFDFFFDFFFDFFFD, 0x30A0305030A03],
                [0x0, 0x3CFFFFAA3CFFFF, 0xEB0000FFEB0000],
                [0x0, 0xFFFF3CFFFFFF3C, 0xEB000000EB],
                [0x0, 0xFFFFFFFFFFFFFF, 0x14551400000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x555555],
                [0x0, 0xCF33CFFFCF33CF, 0x30CC300030CC30],
                [0x0, 0xF3FFF3FFF3FFF3, 0xC000C000C000C],
                [0x0, 0xFFFFFFFFFFFFFF, 0x405040A0405040],
                [0x0, 0xFDFFFDFFFDFFFD, 0x30A0305030A03],
                [0x0, 0x7DFFFFAA7DFFFF, 0xEB0000FFEB0000],
                [0x0, 0xFFFF3CFFFFFF3C, 0xEB000000EB],
                [0x0, 0xFFFFFFFFFFFFFF, 0x14551400000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x550055],
                [0x0, 0xCF33CFFFCF33CF, 0x30CC300030CC30],
                [0x0, 0xF3CCF3FFF3CCF3, 0xC330C000C330C],
                [0x0, 0xFFAFFFFFFFAFFF, 0x40F040A040F040],
                [0x0, 0xFDFFFDFFFDFFFD, 0x30A0305030A03],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55000000],
                [0x0, 0xFFFF3CFFFFFF3C, 0xEB000000EB],
                [0x0, 0xFFFFFFFFFFFFFF, 0x14551400000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x550055],
                [0x0, 0xCF33CFFFCF33CF, 0x30CC300030CC30],
                [0x0, 0xF3CCF3FFF3CCF3, 0xC330C000C330C],
                [0x0, 0xFFAFFFFFFFAFFF, 0x40F040A040F040],
                [0x0, 0xFDFFFDFFFDFFFD, 0x30A0305030A03],
                [0x0, 0xD7FFFF00D7FFFF, 0xBE0000FFBE0000],
                [0x0, 0xFFFF3CFFFFFF3C, 0xEB000000EB],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55555500000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x550055],
                [0x0, 0xCFFFCFFFCFFFCF, 0x30003000300030],
                [0x0, 0xF3CCF3FFF3CCF3, 0xC330C000C330C],
                [0x0, 0xFFAFFFFFFFAFFF, 0x40F040A040F040],
                [0x0, 0xFCFFFCFFFCFFFC, 0x3000300030003],
                [0x0, 0xD7FFFF00D7FFFF, 0xBE0000FFBE0000],
                [0x0, 0xFFFF7DFFFFFF7D, 0xEB000000EB],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55555500000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x555555],
                [0x0, 0xFFFFFF33FFFFFF, 0xCC000000],
                [0x0, 0xF3CCF3FFF3CCF3, 0xC330C000C330C],
                [0x0, 0x3FFF3FFF3FFF3F, 0xC000C000C000C0],
                [0x0, 0xFCFFFCFFFCFFFC, 0x3000300030003],
                [0x0, 0xD7FFFF00D7FFFF, 0xBE0000FFBE0000],
                [0x0, 0xFFFF7DFFFFFF7D, 0xEB000000EB],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x410041],
                [0x0, 0xFF33FF33FF33FF, 0xCC00CC00CC00],
                [0x0, 0xF3FFF3FFF3FFF3, 0xC000C000C000C],
                [0x0, 0xFFFFFFFFFFFFFF, 0x405040A0405040],
                [0x0, 0xFFFFFFFFFFFFFF, 0x105010A010501],
                [0x0, 0x7DFFFFAA7DFFFF, 0xEB0000FFEB0000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55555500000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x145514],
                [0x0, 0xFF33FF33FF33FF, 0xCC00CC00CC00],
                [0x0, 0xFFFFFFCCFFFFFF, 0x33000000],
                [0x0, 0x7FFF7FAF7FFF7F, 0xC0A0C0F0C0A0C0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x105010A010501],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x550000000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x145514],
                [0x0, 0xCFFFCFFFCFFFCF, 0x30003000300030],
                [0x0, 0xFFFFFFCCFFFFFF, 0x33000000],
                [0x0, 0x3FFF3FFF3FFF3F, 0xC000C000C000C0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xC3FFFF00C3FFFF, 0xBE0000FFBE0000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55555500000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x555555],
                [0x0, 0xFFFFFF33FFFFFF, 0xCC000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0x7FFF7FFF7FFF7F, 0xC0A0C050C0A0C0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0x3CFFFFAA3CFFFF, 0xEB0000FFEB0000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55005500000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x410041],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0x7FFF7FFF7FFF7F, 0xC0A0C050C0A0C0],
                [0x0, 0xFCFFFCFFFCFFFC, 0x3000300030003],
                [0x0, 0x3CFFFFAA3CFFFF, 0xEB0000FFEB0000],
                [0x0, 0xFFFF7DFFFFFF7D, 0xEB000000EB],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55005500000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xCF33CF33CF33CF, 0x30CC30CC30CC30],
                [0x0, 0xF3FFF3FFF3FFF3, 0xC000C000C000C],
                [0x0, 0x3FFF3FFF3FFF3F, 0xC000C000C000C0],
                [0x0, 0xFCFFFCFFFCFFFC, 0x3000300030003],
                [0x0, 0x7DFFFFAA7DFFFF, 0xEB0000FFEB0000],
                [0x0, 0xFFFF7DFFFFFF7D, 0xEB000000EB],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55005500000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x5500],
                [0x0, 0xCFFFCFFFCFFFCF, 0x30003000300030],
                [0x0, 0xFFFFFFCCFFFFFF, 0x33000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x405040A0405040],
                [0x0, 0xFFFFFFFFFFFFFF, 0x105010A010501],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x41004100000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x555555],
                [0x0, 0xCF33CFFFCF33CF, 0x30CC300030CC30],
                [0x0, 0xFFFFFFCCFFFFFF, 0x33000000],
                [0x0, 0x7FFF7FAF7FFF7F, 0xC0A0C0F0C0A0C0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x105010A010501],
                [0x0, 0xC3FFFF00C3FFFF, 0xBE0000FFBE0000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x41004100000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x550055],
                [0x0, 0xFFFFFF33FFFFFF, 0xCC000000],
                [0x0, 0xFFCCFFCCFFCCFF, 0x330033003300],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFAFFFFFFFAFF, 0x10F010A010F01],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFD7FFFFFFD7, 0xBE000000BE],
                [0x0, 0xFFFFFFFFFFFFFF, 0x550000000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x410041],
                [0x0, 0xCF33CF33CF33CF, 0x30CC30CC30CC30],
                [0x0, 0xFFCCFFCCFFCCFF, 0x330033003300],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFAFFFFFFFAFF, 0x10F010A010F01],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFD7FFFFFFD7, 0xBE000000BE],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x5500],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFCCFFCCFFCCFF, 0x330033003300],
                [0x0, 0x3FFF3FFF3FFF3F, 0xC000C000C000C0],
                [0x0, 0xFFFAFFFFFFFAFF, 0x10F010A010F01],
                [0x0, 0x7DFFFFAA7DFFFF, 0xEB0000FFEB0000],
                [0x0, 0xFFFFD7FFFFFFD7, 0xBE000000BE],
                [0x0, 0xFFFFFFFFFFFFFF, 0x41004100000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFF33FFFFFF, 0xCC000000],
                [0x0, 0xF3CCF3CCF3CCF3, 0xC330C330C330C],
                [0x0, 0xFFFFFFFFFFFFFF, 0x405040A0405040],
                [0x0, 0xFDFFFDFAFDFFFD, 0x30A030F030A03],
                [0x0, 0xFFFFFFFFFFFFFF, 0x55000000],
                [0x0, 0xFFFFC3FFFFFFC3, 0xBE000000BE],
                [0x0, 0xFFFFFFFFFFFFFF, 0x41004100000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x410041],
                [0x0, 0xFF33FF33FF33FF, 0xCC00CC00CC00],
                [0x0, 0xF3CCF3CCF3CCF3, 0xC330C330C330C],
                [0x0, 0xFFAFFFFFFFAFFF, 0x40F040A040F040],
                [0x0, 0xFDFFFDFAFDFFFD, 0x30A030F030A03],
                [0x0, 0xD7FFFF00D7FFFF, 0xBE0000FFBE0000],
                [0x0, 0xFFFFC3FFFFFFC3, 0xBE000000BE],
                [0x0, 0xFFFFFFFFFFFFFF, 0x14551400000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x145514],
                [0x0, 0xCF33CF33CF33CF, 0x30CC30CC30CC30],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0x7FFF7FAF7FFF7F, 0xC0A0C0F0C0A0C0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xC3FFFF00C3FFFF, 0xBE0000FFBE0000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x550000000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x5500],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xF3CCF3CCF3CCF3, 0xC330C330C330C],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFDFFFDFAFDFFFD, 0x30A030F030A03],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFC3FFFFFFC3, 0xBE000000BE],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFFFFFFFF, 0x0],
                [0x0, 0xCF33CF33CF33CF, 0x30CC30CC30CC30],
                [0x0, 0xF3CCF3CCF3CCF3, 0xC330C330C330C],
                [0x0, 0x7FFF7FAF7FFF7F, 0xC0A0C0F0C0A0C0],
                [0x0, 0xFDFFFDFAFDFFFD, 0x30A030F030A03],
                [0x0, 0xC3FFFF00C3FFFF, 0xBE0000FFBE0000],
                [0x0, 0xFFFFC3FFFFFFC3, 0xBE000000BE],
                [0x0, 0xFFFFFFFFFFFFFF, 0x550000000000],
                [0x0, 0xFFFFFFFFFFFFFF, 0x5500],
            ][twist.to_index()],
        );
        let e_op = apply_permutation_lut!(u64, e_op, twist, [
            4 => [(&0xFFFF0000FFFF<<0)|(&0x30000000000000<<2)|(&0xC00000<<4)|(&0x30000<<8)|(&0xC0000000<<22)|(&0xC000000<<24)|(&0x300000<<28)|(&0xC000000000000<<36)|(&0xC0000000000000<<40)|(&0x3000000000000<<44)|(&0x30000000<<56)|(&0x3000000<<58)|(&0xC0000<<62)],
            6 => [(&0xFFFFFFFF<<0)|(&0xC00000000000<<2)|(&0x300000000<<4)|(&0x3000000000<<6)|(&0x300000000000<<10)|(&0xC00000000<<12)|(&0xC000000000<<14)|(&0x3000000000000<<50)|(&0x30000000000000<<52)|(&0xC0000000000<<54)|(&0xC000000000000<<58)|(&0xC0000000000000<<60)|(&0x30000000000<<62)],
            7 => [(&0xFFFFFFFF000000<<0)|(&0xC0<<2)|(&0xC0000<<4)|(&0x3000<<6)|(&0x3<<10)|(&0x300<<12)|(&0xC<<14)|(&0x300000<<50)|(&0xC000<<52)|(&0xC00000<<54)|(&0xC00<<58)|(&0x30<<60)|(&0x30000<<62)],
            12 => [(&0xFFFF0000FFFF<<0)|(&0xC00000<<2)|(&0xC000000000000<<4)|(&0x30000<<10)|(&0x30000000<<22)|(&0x3000000<<28)|(&0xC0000<<30)|(&0x30000000000000<<34)|(&0xC0000000000000<<38)|(&0x3000000000000<<46)|(&0xC0000000<<52)|(&0xC000000<<58)|(&0x300000<<60)],
            14 => [(&0xFFFFFFFF<<0)|(&0x30000000000000<<2)|(&0xC000000000<<4)|(&0xC00000000000<<6)|(&0xC0300000000<<8)|(&0x3000000000<<12)|(&0xC000000000000<<52)|(&0xC0300000000000<<56)|(&0x30000000000<<58)|(&0x3000000000000<<60)|(&0xC00000000<<62)],
            15 => [(&0xFFFFFFFF000000<<0)|(&0xC<<2)|(&0xC00<<6)|(&0xC0C0<<8)|(&0x3<<10)|(&0x30<<16)|(&0xC0000<<48)|(&0xC00000<<54)|(&0x30300<<56)|(&0x3000<<58)|(&0x300000<<62)],
            20 => [(&0xFFFF0000FFFF<<0)|(&0xC0000<<2)|(&0xC00000<<8)|(&0x30000<<10)|(&0xC000000<<22)|(&0xC0000000<<24)|(&0xC000000300000<<32)|(&0xC0000000000000<<38)|(&0x3000000000000<<40)|(&0x3000000<<56)|(&0x30000000<<58)|(&0x30000000000000<<62)],
            22 => [(&0xFFFFFFFF<<0)|(&0x3000000000<<2)|(&0x30C000000000<<8)|(&0x30000000000<<10)|(&0x300000000<<12)|(&0xC00000000<<20)|(&0x30000000000000<<44)|(&0xC0000000000000<<52)|(&0xC00000000000<<54)|(&0x30C0000000000<<56)|(&0xC000000000000<<62)],
            23 => [(&0xFFFFFFFF000000<<0)|(&0x30000<<2)|(&0x30C<<8)|(&0x30<<10)|(&0xC00<<12)|(&0x3<<20)|(&0xC00000<<44)|(&0x3000<<52)|(&0xC0000<<54)|(&0x30C000<<56)|(&0xC0<<62)],
            28 => [(&0xFFFF0000FFFF<<0)|(&0x3000030030000<<2)|(&0xC0000030C0000<<4)|(&0x300000C0300000<<60)|(&0xC000000CC00000<<62)],
            30 => [(&0xFFFFFFFF<<0)|(&0xC00C00000000<<4)|(&0xC000000000<<6)|(&0xC0000000000<<10)|(&0x300000000<<14)|(&0x3000000000<<18)|(&0xC000000000000<<46)|(&0xC0000000000000<<50)|(&0x300000000000<<54)|(&0x3000000000000<<58)|(&0x30030000000000<<60)],
            31 => [(&0xFFFFFFFF000000<<0)|(&0x3<<2)|(&0x3000<<4)|(&0xC<<6)|(&0xC030<<8)|(&0xC0<<12)|(&0x30000<<52)|(&0xC0300<<56)|(&0x300000<<58)|(&0xC00<<60)|(&0xC00000<<62)],
            36 => [(&0xFFFF0000FFFF<<0)|(&0x3000000300000<<4)|(&0xC0000<<10)|(&0xC000000<<22)|(&0xC0000003000000<<30)|(&0x30000<<34)|(&0x30000000000000<<38)|(&0xC000000000000<<44)|(&0xC0000000<<50)|(&0x30000000<<58)|(&0xC00000<<60)],
            38 => [(&0xC0030FFFFFFFF<<0)|(&0x30C000000000<<4)|(&0x30300000000<<14)|(&0xC00000000<<18)|(&0x30000000000000<<46)|(&0xC0C00000000000<<50)|(&0x30C0000000000<<60)],
            39 => [(&0xFFFFFFFF000000<<0)|(&0x30030<<4)|(&0xC00<<6)|(&0xC<<10)|(&0x300<<14)|(&0x3<<18)|(&0xC00000<<46)|(&0xC000<<50)|(&0x300000<<54)|(&0x3000<<58)|(&0xC00C0<<60)],
            44 => [(&0xFFFF0000FFFF<<0)|(&0x3000000000000<<2)|(&0xC0000<<8)|(&0x300000<<10)|(&0x3000000<<24)|(&0xC000000C000000<<28)|(&0x30000<<36)|(&0xC000000000000<<38)|(&0x30000000000000<<40)|(&0x30000000<<52)|(&0xC0000000<<56)|(&0xC00000<<62)],
            46 => [(&0x30000CFFFFFFFF<<0)|(&0xC0C000000000<<2)|(&0xC0300000000<<12)|(&0x3000000000<<14)|(&0xC000000000000<<50)|(&0xC0300000000000<<52)|(&0x3030000000000<<62)],
            47 => [(&0xFFFFFFFF30000C<<0)|(&0xC0C0<<2)|(&0xC03<<12)|(&0x30<<14)|(&0xC0000<<50)|(&0xC03000<<52)|(&0x30300<<62)],
            52 => [(&0xFFFF0000FFFF<<0)|(&0x30000<<2)|(&0xC0000<<6)|(&0x300000<<8)|(&0x30000000<<20)|(&0xC0000000<<24)|(&0xC00000<<28)|(&0x3000000000000<<36)|(&0xC000000000000<<40)|(&0x30000000000000<<42)|(&0x3000000<<56)|(&0xC000000<<60)|(&0xC0000000000000<<62)],
            54 => [(&0xFFFFFFFF<<0)|(&0x330000000000<<2)|(&0x3000000000<<14)|(&0xC300000000<<16)|(&0xC00000000<<18)|(&0x30000000000000<<46)|(&0xC3000000000000<<48)|(&0xC000000000000<<50)|(&0xCC0000000000<<62)],
            55 => [(&0xFFFFFFFF000000<<0)|(&0x3003C<<6)|(&0xF00<<10)|(&0x3<<16)|(&0xC00000<<48)|(&0xF000<<54)|(&0x3C00C0<<58)],
            60 => [(&0x30FFFF000CFFFF<<0)|(&0xC00000<<2)|(&0x30000<<12)|(&0xC0000000<<18)|(&0xC000000<<28)|(&0x300000<<30)|(&0xC000000000000<<34)|(&0xC0000000000000<<36)|(&0x3000000000000<<46)|(&0x30000000<<52)|(&0x3000000<<62)],
            61 => [(&0xFFFF00FFFFFF00<<0)|(&0xF0000000F0<<28)|(&0xF0000000F<<36)],
            62 => [(&0xFFFFFFFF<<0)|(&0xC000000000000<<2)|(&0xC000000000<<6)|(&0x30300000000<<8)|(&0x300000000000<<10)|(&0xC00000000<<16)|(&0x30000000000000<<48)|(&0xC0000000000<<54)|(&0xC0C00000000000<<56)|(&0x3000000000000<<58)|(&0x3000000000<<62)],
            63 => [(&0xFFFFFFFF000000<<0)|(&0xC00C<<4)|(&0xC0<<6)|(&0xC00<<10)|(&0x3<<14)|(&0x30<<18)|(&0xC0000<<46)|(&0xC00000<<50)|(&0x3000<<54)|(&0x30000<<58)|(&0x300300<<60)],
            68 => [(&0xFFFFC300FFFF<<0)|(&0xC000000<<2)|(&0xCC000000CC0000<<30)|(&0x33000000330000<<34)|(&0x30000000<<62)],
            69 => [(&0xFFFF00FFFFFF00<<0)|(&0xCC000000CC<<30)|(&0x3300000033<<34)],
            70 => [(&0xFFFFFFFF<<0)|(&0x3300300000000<<2)|(&0xC030C00000000<<4)|(&0x30C03000000000<<60)|(&0xC00CC000000000<<62)],
            71 => [(&0xFFFFFFFF000000<<0)|(&0x30<<2)|(&0x30C0<<8)|(&0x300<<10)|(&0x3<<12)|(&0xC<<20)|(&0x300000<<44)|(&0xC00000<<52)|(&0xC000<<54)|(&0x30C00<<56)|(&0xC0000<<62)],
            76 => [(&0xFFFF0000FFFF<<0)|(&0x30000003C0000<<6)|(&0xF000000<<26)|(&0xC0000000030000<<32)|(&0x3C000000000000<<42)|(&0xF0000000<<54)|(&0xC00000<<58)],
            77 => [(&0xFFFF00FFFFFF00<<0)|(&0x3000000030<<30)|(&0xC3000000C3<<32)|(&0xC0000000C<<34)],
            78 => [(&0xFFFFFFFF<<0)|(&0xC00000000<<2)|(&0xC0000000000<<6)|(&0xC0C000000000<<8)|(&0x300000000<<10)|(&0x3000000000<<16)|(&0xC000000000000<<48)|(&0xC0000000000000<<54)|(&0x3030000000000<<56)|(&0x300000000000<<58)|(&0x30000000000000<<62)],
            79 => [(&0xFFFFFFFF0C0030<<0)|(&0x30C0<<4)|(&0x303<<14)|(&0xC<<18)|(&0x300000<<46)|(&0xC0C000<<50)|(&0x30C00<<60)],
            84 => [(&0xFFFF0000FFFF<<0)|(&0x300000<<2)|(&0xC00000<<8)|(&0x30000<<12)|(&0x30000000<<24)|(&0x3000000<<26)|(&0x30000000000000<<28)|(&0xC00000000C0000<<36)|(&0x3000000000000<<40)|(&0xC0000000<<54)|(&0xC000000<<56)|(&0xC000000000000<<62)],
            85 => [(&0xFFFF00FFFFFF00<<0)|(&0xC0000000C<<2)|(&0x300000003<<6)|(&0xC0000000C0<<58)|(&0x3000000030<<62)],
            86 => [(&0xFFFFFFFF<<0)|(&0x3003000000000<<4)|(&0xC0000000000<<6)|(&0xC00000000<<10)|(&0x30000000000<<14)|(&0x300000000<<18)|(&0xC0000000000000<<46)|(&0xC00000000000<<50)|(&0x30000000000000<<54)|(&0x300000000000<<58)|(&0xC00C000000000<<60)],
            87 => [(&0xFFFFFFFF0300C0<<0)|(&0xC30<<8)|(&0x30C<<12)|(&0x3<<22)|(&0xC00000<<42)|(&0x30C000<<52)|(&0xC3000<<56)],
            92 => [(&0xFFFF0000FFFF<<0)|(&0xC0000<<4)|(&0xC00000<<6)|(&0x30000<<14)|(&0xC0000000<<20)|(&0xC000000<<26)|(&0xC000000000000<<30)|(&0xC0000000300000<<34)|(&0x3000000000000<<42)|(&0x30000000<<54)|(&0x30000003000000<<60)],
            94 => [(&0xFFFFFFFF<<0)|(&0x3000000000000<<2)|(&0x30C00000000<<8)|(&0x3000000000<<10)|(&0xC0000000000<<12)|(&0x300000000<<20)|(&0xC0000000000000<<44)|(&0x300000000000<<52)|(&0xC000000000000<<54)|(&0x30C00000000000<<56)|(&0xC000000000<<62)],
            95 => [(&0xFFFFFFFF000000<<0)|(&0x3300<<2)|(&0x30<<14)|(&0xC3<<16)|(&0xC<<18)|(&0x300000<<46)|(&0xC30000<<48)|(&0xC0000<<50)|(&0xCC00<<62)],
            100 => [(&0xCFFFF0030FFFF<<0)|(&0xC00000<<4)|(&0x30000<<14)|(&0x30000000<<20)|(&0x30000003000000<<30)|(&0xC00000000C0000<<34)|(&0x3000000000000<<44)|(&0xC0000000<<50)|(&0xC000000<<60)],
            102 => [(&0xFFFFFFFF<<0)|(&0x300000000<<2)|(&0x300000000000<<4)|(&0xC00000000<<6)|(&0xC03000000000<<8)|(&0xC000000000<<12)|(&0x3000000000000<<52)|(&0xC030000000000<<56)|(&0x30000000000000<<58)|(&0xC0000000000<<60)|(&0xC0000000000000<<62)],
            103 => [(&0xFFFFFFFF000000<<0)|(&0x300330<<2)|(&0x30C03<<4)|(&0xC030C0<<60)|(&0xCC00C<<62)],
            108 => [(&0xFFFF3C00FFFF<<0)|(&0x3000000<<6)|(&0xF0000000F00000<<28)|(&0xF0000000F0000<<36)|(&0xC0000000<<58)],
            109 => [(&0xFFFF00FFFFFF00<<0)|(&0x3000000030<<2)|(&0x300000003<<4)|(&0xC0000000C0<<60)|(&0xC0000000C<<62)],
            110 => [(&0x300C0FFFFFFFF<<0)|(&0xC3000000000<<8)|(&0x30C00000000<<12)|(&0x300000000<<22)|(&0xC0000000000000<<42)|(&0x30C00000000000<<52)|(&0xC300000000000<<56)],
            111 => [(&0xFFFFFFFF000000<<0)|(&0xC0000<<2)|(&0xC0<<6)|(&0x303<<8)|(&0x3000<<10)|(&0xC<<16)|(&0x300000<<48)|(&0xC00<<54)|(&0xC0C000<<56)|(&0x30000<<58)|(&0x30<<62)],
            116 => [(&0x3FFFF00C0FFFF<<0)|(&0x300000<<8)|(&0xC0000<<12)|(&0xC000000<<24)|(&0xC0000000000000<<26)|(&0x3000000<<28)|(&0x30000000000000<<36)|(&0x30000<<38)|(&0xC000000000000<<40)|(&0xC0000000<<52)|(&0x30000000<<56)],
            117 => [(&0xFFFF00FFFFFF00<<0)|(&0x300000003<<2)|(&0xC0000000C<<4)|(&0x3000000030<<60)|(&0xC0000000C0<<62)],
            118 => [(&0xC00003FFFFFFFF<<0)|(&0xC03000000000<<4)|(&0x300C00000000<<8)|(&0xC000000000<<10)|(&0x3000000000000<<54)|(&0x300C0000000000<<56)|(&0xC030000000000<<60)],
            119 => [(&0xFFFFFFFF000000<<0)|(&0xC000<<2)|(&0x3<<4)|(&0x30<<6)|(&0x3000<<10)|(&0xC<<12)|(&0xC0<<14)|(&0x30000<<50)|(&0x300000<<52)|(&0xC00<<54)|(&0xC0000<<58)|(&0xC00000<<60)|(&0x300<<62)],
            124 => [(&0xFFFF0000FFFF<<0)|(&0x33000000<<2)|(&0x30000000300000<<30)|(&0xC3000000C30000<<32)|(&0xC0000000C0000<<34)|(&0xCC000000<<62)],
            126 => [(&0xC300FFFFFFFF<<0)|(&0xC0000000000<<2)|(&0xCC00000000<<14)|(&0x3300000000<<18)|(&0xCC000000000000<<46)|(&0x33000000000000<<50)|(&0x300000000000<<62)],
            127 => [(&0xFFFFFFFFC00003<<0)|(&0xC030<<4)|(&0x300C<<8)|(&0xC0<<10)|(&0x30000<<54)|(&0x300C00<<56)|(&0xC0300<<60)],
            132 => [(&0xFFFF0000FFFF<<0)|(&0xC00000C0C0000<<2)|(&0x3000003030000<<6)|(&0xC00000C0C00000<<58)|(&0x30000030300000<<62)],
            134 => [(&0xFFFFFFFF<<0)|(&0xF0000000000<<4)|(&0xC000000000<<10)|(&0x3C00000000<<16)|(&0x300000000<<22)|(&0xC0000000000000<<42)|(&0x3C000000000000<<48)|(&0x3000000000000<<54)|(&0xF00000000000<<60)],
            135 => [(&0xFFFFFFFF003C00<<0)|(&0x300<<6)|(&0xF0<<12)|(&0xF<<20)|(&0xF00000<<44)|(&0xF0000<<52)|(&0xC000<<58)],
            140 => [(&0xFFFF0000FFFF<<0)|(&0xC000000000000<<2)|(&0xC00000<<6)|(&0x30000<<8)|(&0x3000000<<24)|(&0x30000000<<26)|(&0x300000000C0000<<32)|(&0xC0000000000000<<40)|(&0x3000000000000<<42)|(&0xC000000<<54)|(&0xC0000000<<56)|(&0x300000<<62)],
            142 => [(&0xFFFFFFFF<<0)|(&0x3003C00000000<<6)|(&0xF0000000000<<10)|(&0x300000000<<16)|(&0xC0000000000000<<48)|(&0xF00000000000<<54)|(&0x3C00C000000000<<58)],
            143 => [(&0xFFFFFFFF000000<<0)|(&0xF00<<4)|(&0xC0<<10)|(&0x3C<<16)|(&0x3<<22)|(&0xC00000<<42)|(&0x3C0000<<48)|(&0x30000<<54)|(&0xF000<<60)],
            148 => [(&0xFFFF0000FFFF<<0)|(&0x30000003300000<<2)|(&0x300000C030000<<4)|(&0xC0000030C00000<<60)|(&0xC0000C00C0000<<62)],
            150 => [(&0xFFFFFFFF<<0)|(&0xF00300000000<<6)|(&0x3C00000000<<10)|(&0xC000000000<<16)|(&0x3000000000000<<48)|(&0x3C000000000000<<54)|(&0xC00F0000000000<<58)],
            151 => [(&0xFFFFFFFF000000<<0)|(&0xF003<<6)|(&0x3C<<10)|(&0xC0<<16)|(&0x30000<<48)|(&0x3C0000<<54)|(&0xC00F00<<58)],
            156 => [(&0xFFFF0000FFFF<<0)|(&0x30000<<4)|(&0x300000<<6)|(&0xC0000<<12)|(&0xC0000000<<18)|(&0x30000000<<26)|(&0xC00000<<30)|(&0x3000000000000<<34)|(&0x30000000000000<<36)|(&0xC000000000000<<42)|(&0xC000000<<54)|(&0xC0000000000000<<60)|(&0x3000000<<62)],
            158 => [(&0xFFFFFFFF<<0)|(&0xC000000000<<2)|(&0xC000000000000<<4)|(&0x300000000000<<6)|(&0x300000000<<10)|(&0x30000000000<<12)|(&0xC00000000<<14)|(&0x30000000000000<<50)|(&0xC00000000000<<52)|(&0xC0000000000000<<54)|(&0xC0000000000<<58)|(&0x3000000000<<60)|(&0x3000000000000<<62)],
            159 => [(&0xFFFFFFFF000000<<0)|(&0x300000<<2)|(&0xC0<<4)|(&0xC000<<6)|(&0xC03<<8)|(&0x30<<12)|(&0xC0000<<52)|(&0xC03000<<56)|(&0x300<<58)|(&0x30000<<60)|(&0xC<<62)],
            161 => [(&0xF3CCF3CCF3CCF3<<0)|(&0x3000000<<4)|(&0x300000003000<<28)|(&0xC000C000C000C<<32)|(&0x30000000300<<36)|(&0x30000000<<60)],
            163 => [(&0xFCF0FCF0FCF0FC<<0)|(&0x3000000<<2)|(&0xC0000000C00<<30)|(&0x3000300030003<<32)|(&0x30000000300<<34)|(&0xC000000<<62)],
            164 => [(&0xC0FFFF0003FFFF<<0)|(&0x300000<<4)|(&0xC0000<<8)|(&0xC0000000<<20)|(&0x30000000<<24)|(&0xC00000<<26)|(&0x3000000000000<<38)|(&0x30000000000000<<40)|(&0xC000000000000<<44)|(&0xC000000<<56)|(&0x3000000<<60)],
            165 => [(&0xFFFF00FFFFFF00<<0)|(&0xC0000000C0<<26)|(&0x3C0000003C<<32)|(&0x300000003<<38)],
            166 => [(&0x3C00FFFFFFFF<<0)|(&0x30000000000<<6)|(&0xF000000000<<12)|(&0xF00000000<<20)|(&0xF0000000000000<<44)|(&0xF000000000000<<52)|(&0xC00000000000<<58)],
            167 => [(&0xFFFFFFFF00C300<<0)|(&0xC00<<2)|(&0xCC<<14)|(&0x33<<18)|(&0xCC0000<<46)|(&0x330000<<50)|(&0x3000<<62)],
            168 => [(&0xCF33CF33CF33CF<<0)|(&0xC000000<<4)|(&0xC0000000C000<<28)|(&0x30003000300030<<32)|(&0xC0000000C00<<36)|(&0xC0000000<<60)],
            170 => [(&0x3F0F3F0F3F0F3F<<0)|(&0x30000000<<2)|(&0xC0000000C000<<30)|(&0xC000C000C000C0<<32)|(&0x300000003000<<34)|(&0xC0000000<<62)],
            172 => [(&0xFFFF0000FFFF<<0)|(&0xF000000<<4)|(&0xC0000000C00000<<26)|(&0x3C0000003C0000<<32)|(&0x3000000030000<<38)|(&0xF0000000<<60)],
            174 => [(&0xFFFFFFFF<<0)|(&0xC0C0C00000000<<2)|(&0x3030300000000<<6)|(&0xC0C0C000000000<<58)|(&0x30303000000000<<62)],
            175 => [(&0xFFFFFFFF000000<<0)|(&0xC0C0C<<2)|(&0x30303<<6)|(&0xC0C0C0<<58)|(&0x303030<<62)],
            180 => [(&0xFFFF0000FFFF<<0)|(&0x30000<<6)|(&0x3C0000<<10)|(&0xF0000000<<22)|(&0x3000000C00000<<32)|(&0x3C000000000000<<38)|(&0xC000000F000000<<58)],
            182 => [(&0xFFFFFFFF<<0)|(&0x30033000000000<<2)|(&0x30C0300000000<<4)|(&0xC030C000000000<<60)|(&0xCC00C00000000<<62)],
            183 => [(&0xFFFFFFFF000000<<0)|(&0x33003<<2)|(&0xC030C<<4)|(&0x30C030<<60)|(&0xC00CC0<<62)],
        ]);
        let c_o = crate::lut::update_orientations_u32(
            c_o,
            [
                [0x0, 0x77777777, 0xCCCCCCCC],
                [0x0, 0xDDDDDDDD, 0x33333333],
                [0xA0A0A0A0, 0x5F5F5F5F, 0xF0F0F0F0],
                [0xF0F0F0F, 0xFAFAFAFA, 0xF0F0F0F],
                [0xFF00FF00, 0x55FF55FF, 0xFF00FF00],
                [0x550055, 0xFFAAFFAA, 0xFF00FF],
                [0x55550000, 0x5555FFFF, 0xFFFF0000],
                [0xAAAA, 0xFFFFAAAA, 0xFFFF],
                [0x0, 0xFFFFFFFF, 0x44444444],
                [0x0, 0xFFFFFFFF, 0x11111111],
                [0xA0A0A0A0, 0x5F5F5F5F, 0xF0F0F0F0],
                [0xA0A0A0A, 0xF5F5F5F5, 0xF0F0F0F],
                [0xFF00FF00, 0x55FF55FF, 0xFF00FF00],
                [0xFF00FF, 0xFF55FF55, 0xFF00FF],
                [0xAAAA0000, 0xAAAAFFFF, 0xFFFF0000],
                [0xAAAA, 0xFFFFFFFF, 0xAAAA],
                [0x0, 0xBBBBBBBB, 0xCCCCCCCC],
                [0x0, 0xFFFFFFFF, 0x11111111],
                [0xA0A0A0A0, 0xFFFFFFFF, 0xA0A0A0A0],
                [0xA0A0A0A, 0xF5F5F5F5, 0xF0F0F0F],
                [0xFF00FF00, 0xFF00FF, 0xFF00FF00],
                [0xFF00FF, 0xFF55FF55, 0xFF00FF],
                [0xAAAA0000, 0xAAAAFFFF, 0xFFFF0000],
                [0x5555, 0xFFFF5555, 0xFFFF],
                [0x0, 0xBBBBBBBB, 0xCCCCCCCC],
                [0x0, 0xEEEEEEEE, 0x33333333],
                [0xF0F0F0F0, 0xAFAFAFAF, 0xF0F0F0F0],
                [0xA0A0A0A, 0xF5F5F5F5, 0xF0F0F0F],
                [0x55005500, 0xFFFFFFFF, 0x55005500],
                [0xFF00FF, 0xFF55FF55, 0xFF00FF],
                [0xAAAA0000, 0xAAAAFFFF, 0xFFFF0000],
                [0x5555, 0xFFFF5555, 0xFFFF],
                [0x0, 0xBBBBBBBB, 0xCCCCCCCC],
                [0x0, 0xEEEEEEEE, 0x33333333],
                [0xF0F0F0F0, 0xAFAFAFAF, 0xF0F0F0F0],
                [0xA0A0A0A, 0xF5F5F5F5, 0xF0F0F0F],
                [0x55005500, 0xAAFFAAFF, 0xFF00FF00],
                [0xFF00FF, 0xFF55FF55, 0xFF00FF],
                [0xAAAA0000, 0xFFFFFFFF, 0xAAAA0000],
                [0x5555, 0xFFFF5555, 0xFFFF],
                [0x0, 0xFFFFFFFF, 0x44444444],
                [0x0, 0xEEEEEEEE, 0x33333333],
                [0xF0F0F0F0, 0xAFAFAFAF, 0xF0F0F0F0],
                [0x0, 0xFFFFFFFF, 0x5050505],
                [0x55005500, 0xAAFFAAFF, 0xFF00FF00],
                [0xFF00FF, 0xFF00FF00, 0xFF00FF],
                [0xAAAA0000, 0xFFFFFFFF, 0xAAAA0000],
                [0xAAAA, 0xFFFFFFFF, 0xAAAA],
                [0x0, 0x33333333, 0xCCCCCCCC],
                [0x0, 0xEEEEEEEE, 0x33333333],
                [0x0, 0xFFFFFFFF, 0x50505050],
                [0x0, 0xFFFFFFFF, 0x5050505],
                [0x55005500, 0xAAFFAAFF, 0xFF00FF00],
                [0xFF00FF, 0xFF00FF00, 0xFF00FF],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFF0000, 0xFFFF],
                [0x0, 0x77777777, 0xCCCCCCCC],
                [0x0, 0xFFFFFFFF, 0x11111111],
                [0xA0A0A0A0, 0xFFFFFFFF, 0xA0A0A0A0],
                [0xA0A0A0A, 0xFFFFFFFF, 0xA0A0A0A],
                [0xFF00FF00, 0xFF00FF, 0xFF00FF00],
                [0x550055, 0xFFFFFFFF, 0x550055],
                [0xAAAA0000, 0xFFFFFFFF, 0xAAAA0000],
                [0xAAAA, 0xFFFFAAAA, 0xFFFF],
                [0x0, 0x77777777, 0xCCCCCCCC],
                [0x0, 0xCCCCCCCC, 0x33333333],
                [0xF0F0F0F0, 0xF0F0F0F, 0xF0F0F0F0],
                [0xA0A0A0A, 0xFFFFFFFF, 0xA0A0A0A],
                [0x55005500, 0xFFFFFFFF, 0x55005500],
                [0x550055, 0xFFFFFFFF, 0x550055],
                [0x55550000, 0xFFFFFFFF, 0x55550000],
                [0xAAAA, 0xFFFFAAAA, 0xFFFF],
                [0x0, 0xFFFFFFFF, 0x44444444],
                [0x0, 0xCCCCCCCC, 0x33333333],
                [0x0, 0xFFFFFFFF, 0x50505050],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFF, 0xAA00AA00],
                [0x0, 0xFFFFFFFF, 0x0],
                [0xAAAA0000, 0xFFFFFFFF, 0xAAAA0000],
                [0xAAAA, 0xFFFFFFFF, 0xAAAA],
                [0x0, 0x33333333, 0xCCCCCCCC],
                [0x0, 0xFFFFFFFF, 0x0],
                [0xA0A0A0A0, 0x5F5F5F5F, 0xF0F0F0F0],
                [0x0, 0xFFFFFFFF, 0x0],
                [0xFF00FF00, 0x55FF55FF, 0xFF00FF00],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x55550000, 0x5555FFFF, 0xFFFF0000],
                [0x0, 0xFFFF0000, 0xFFFF],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFF, 0x0],
                [0xA0A0A0A0, 0x5F5F5F5F, 0xF0F0F0F0],
                [0x0, 0xFFFFFFFF, 0x5050505],
                [0xFF00FF00, 0x55FF55FF, 0xFF00FF00],
                [0xFF00FF, 0xFF00FF00, 0xFF00FF],
                [0x55550000, 0x5555FFFF, 0xFFFF0000],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFF, 0x88888888],
                [0x0, 0xFFFFFFFF, 0x11111111],
                [0x0, 0xFFFFFFFF, 0x50505050],
                [0x0, 0xFFFFFFFF, 0x5050505],
                [0xFF00FF00, 0xFF00FF, 0xFF00FF00],
                [0xFF00FF, 0xFF00FF00, 0xFF00FF],
                [0x55550000, 0x5555FFFF, 0xFFFF0000],
                [0x5555, 0xFFFFFFFF, 0x5555],
                [0x0, 0xFFFFFFFF, 0x44444444],
                [0x0, 0xCCCCCCCC, 0x33333333],
                [0xA0A0A0A0, 0xFFFFFFFF, 0xA0A0A0A0],
                [0xA0A0A0A, 0xFFFFFFFF, 0xA0A0A0A],
                [0x55005500, 0xFFFFFFFF, 0x55005500],
                [0x550055, 0xFFFFFFFF, 0x550055],
                [0x0, 0xFFFF, 0xFFFF0000],
                [0xAAAA, 0xFFFFFFFF, 0xAAAA],
                [0x0, 0xBBBBBBBB, 0xCCCCCCCC],
                [0x0, 0xCCCCCCCC, 0x33333333],
                [0xF0F0F0F0, 0xF0F0F0F, 0xF0F0F0F0],
                [0xA0A0A0A, 0xFFFFFFFF, 0xA0A0A0A],
                [0x0, 0xFFFFFFFF, 0xAA00AA00],
                [0x550055, 0xFFFFFFFF, 0x550055],
                [0x0, 0xFFFF, 0xFFFF0000],
                [0x5555, 0xFFFF5555, 0xFFFF],
                [0x0, 0x33333333, 0xCCCCCCCC],
                [0x0, 0xDDDDDDDD, 0x33333333],
                [0x0, 0xFFFFFFFF, 0x0],
                [0xF0F0F0F, 0xFAFAFAFA, 0xF0F0F0F],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x550055, 0xFFAAFFAA, 0xFF00FF],
                [0x55550000, 0xFFFFFFFF, 0x55550000],
                [0x0, 0xFFFF0000, 0xFFFF],
                [0x0, 0xFFFFFFFF, 0x88888888],
                [0x0, 0xDDDDDDDD, 0x33333333],
                [0x0, 0xFFFFFFFF, 0x0],
                [0xF0F0F0F, 0xFAFAFAFA, 0xF0F0F0F],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x550055, 0xFFAAFFAA, 0xFF00FF],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x5555, 0xFFFFFFFF, 0x5555],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xDDDDDDDD, 0x33333333],
                [0x0, 0xFFFFFFFF, 0x50505050],
                [0xF0F0F0F, 0xFAFAFAFA, 0xF0F0F0F],
                [0xFF00FF00, 0xFF00FF, 0xFF00FF00],
                [0x550055, 0xFFAAFFAA, 0xFF00FF],
                [0x0, 0xFFFF, 0xFFFF0000],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0x33333333, 0xCCCCCCCC],
                [0x0, 0xFFFFFFFF, 0x22222222],
                [0xA0A0A0A0, 0xFFFFFFFF, 0xA0A0A0A0],
                [0xF0F0F0F, 0xF0F0F0F0, 0xF0F0F0F],
                [0x55005500, 0xFFFFFFFF, 0x55005500],
                [0x0, 0xFFFFFFFF, 0xAA00AA],
                [0x0, 0xFFFF, 0xFFFF0000],
                [0x0, 0xFFFF0000, 0xFFFF],
                [0x0, 0x77777777, 0xCCCCCCCC],
                [0x0, 0xFFFFFFFF, 0x22222222],
                [0xF0F0F0F0, 0xAFAFAFAF, 0xF0F0F0F0],
                [0xF0F0F0F, 0xF0F0F0F0, 0xF0F0F0F],
                [0x55005500, 0xAAFFAAFF, 0xFF00FF00],
                [0x0, 0xFFFFFFFF, 0xAA00AA],
                [0xAAAA0000, 0xAAAAFFFF, 0xFFFF0000],
                [0xAAAA, 0xFFFFAAAA, 0xFFFF],
                [0x0, 0xFFFFFFFF, 0x88888888],
                [0x0, 0xFFFFFFFF, 0x0],
                [0xF0F0F0F0, 0xF0F0F0F, 0xF0F0F0F0],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFF, 0xAA00AA00],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x55550000, 0xFFFFFFFF, 0x55550000],
                [0x5555, 0xFFFFFFFF, 0x5555],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFF, 0x22222222],
                [0x0, 0xFFFFFFFF, 0x0],
                [0xF0F0F0F, 0xF0F0F0F0, 0xF0F0F0F],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFF, 0xAA00AA],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFF, 0x0],
                [0x0, 0xFFFFFFFF, 0x88888888],
                [0x0, 0xFFFFFFFF, 0x22222222],
                [0xF0F0F0F0, 0xF0F0F0F, 0xF0F0F0F0],
                [0xF0F0F0F, 0xF0F0F0F0, 0xF0F0F0F],
                [0x0, 0xFFFFFFFF, 0xAA00AA00],
                [0x0, 0xFFFFFFFF, 0xAA00AA],
                [0x55550000, 0xFFFFFFFF, 0x55550000],
                [0x5555, 0xFFFFFFFF, 0x5555],
            ][twist.to_index()],
        );
        let c_o = apply_permutation_lut!(u32, c_o, twist, [
            0 => [(&0x333FF333<<0)|(&0xC0<<4)|(&0xC0000000<<8)|(&0xC00000<<12)|(&0xC00<<20)|(&0xC<<24)|(&0xC000000<<28)],
            1 => [(&0xFCCCCCCF<<0)|(&0x300000<<4)|(&0x3000<<8)|(&0x30<<12)|(&0x3000000<<20)|(&0x30000<<24)|(&0x300<<28)],
            2 => [(&0x3F0F0FCF<<0)|(&0xC000<<8)|(&0x30<<10)|(&0xC00000<<14)|(&0x3000<<18)|(&0xC0000000<<22)|(&0x300000<<24)],
            3 => [(&0xF3F0F0FC<<0)|(&0x300<<8)|(&0x30000<<10)|(&0xC000000<<14)|(&0x3<<18)|(&0xC00<<22)|(&0xC0000<<24)],
            4 => [(&0xC0FF03FF<<0)|(&0xC000000<<2)|(&0xC000<<12)|(&0xC00<<14)|(&0x30000000<<18)|(&0x3000000<<20)|(&0x3000<<30)],
            5 => [(&0xFF30FF0C<<0)|(&0x3<<6)|(&0xC0<<12)|(&0xC0000<<14)|(&0x30<<18)|(&0x30000<<20)|(&0xC00000<<26)],
            6 => [(&0x300CFFFF<<0)|(&0xC00000<<4)|(&0x30000<<6)|(&0x300000<<10)|(&0xC000000<<22)|(&0xC0000000<<26)|(&0x3000000<<28)],
            7 => [(&0xFFFF300C<<0)|(&0x30<<4)|(&0x300<<6)|(&0x3<<10)|(&0xC000<<22)|(&0xC0<<26)|(&0xC00<<28)],
            8 => [(&0x33333333<<0)|(&0xCC0000<<8)|(&0xCC0000CC<<16)|(&0xCC00<<24)],
            9 => [(&0xCCCCCCCC<<0)|(&0x3003003<<4)|(&0x300<<12)|(&0x300000<<20)|(&0x30030030<<28)],
            10 => [(&0xCF0F0F3F<<0)|(&0xC0<<6)|(&0x3000<<8)|(&0xC000<<14)|(&0x300000<<18)|(&0xC00000<<24)|(&0x30000000<<26)],
            11 => [(&0xF0FCF3F0<<0)|(&0xC000000<<8)|(&0x30000<<10)|(&0xC<<14)|(&0x3000000<<18)|(&0xC00<<22)|(&0x3<<24)],
            12 => [(&0x30FF0CFF<<0)|(&0x3000000<<6)|(&0x3000<<12)|(&0xC0000000<<14)|(&0x300<<18)|(&0xC000000<<20)|(&0xC000<<26)],
            13 => [(&0xFF0CFF30<<0)|(&0x3<<6)|(&0x300000<<12)|(&0xC0<<14)|(&0x30000<<18)|(&0xC<<20)|(&0xC00000<<26)],
            14 => [(&0xC003FFFF<<0)|(&0xC000000<<2)|(&0xC00000<<4)|(&0xC0000<<6)|(&0x30000000<<26)|(&0x3000000<<28)|(&0x300000<<30)],
            15 => [(&0xFFFF0000<<0)|(&0x33<<2)|(&0xCC<<8)|(&0x3300<<24)|(&0xCC00<<30)],
            16 => [(&0x33F33F33<<0)|(&0xC0000000<<8)|(&0xC00C0<<12)|(&0xC00C000<<20)|(&0xC<<24)],
            17 => [(&0xCCCCCCCC<<0)|(&0x33<<8)|(&0x333300<<16)|(&0x33000000<<24)],
            18 => [(&0xF0F0F0F<<0)|(&0x300030<<2)|(&0xC000C0<<8)|(&0x30003000<<24)|(&0xC000C000<<30)],
            19 => [(&0xF3F0F0FC<<0)|(&0xC00<<8)|(&0x3<<10)|(&0xC0000<<14)|(&0x300<<18)|(&0xC000000<<22)|(&0x30000<<24)],
            20 => [(&0xFF00FF<<0)|(&0x3300<<2)|(&0x3300CC00<<16)|(&0xCC000000<<30)],
            21 => [(&0xFF30FF0C<<0)|(&0x30000<<6)|(&0x30<<12)|(&0xC00000<<14)|(&0x3<<18)|(&0xC0000<<20)|(&0xC0<<26)],
            22 => [(&0x3C0FFFF<<0)|(&0xC0000<<2)|(&0x300000<<10)|(&0x30000<<12)|(&0xC0000000<<20)|(&0xC000000<<22)|(&0x30000000<<30)],
            23 => [(&0xFFFF03C0<<0)|(&0xC00<<2)|(&0x3<<10)|(&0xC<<12)|(&0x3000<<20)|(&0xC000<<22)|(&0x30<<30)],
            24 => [(&0xF333333F<<0)|(&0xC0<<4)|(&0xC00<<8)|(&0xC000<<12)|(&0xC0000<<20)|(&0xC00000<<24)|(&0xC000000<<28)],
            25 => [(&0xCCCFFCCC<<0)|(&0x300000<<4)|(&0x3000000<<8)|(&0x30000000<<12)|(&0x3<<20)|(&0x30<<24)|(&0x300<<28)],
            26 => [(&0xFCF3F0F<<0)|(&0x30000000<<8)|(&0x30<<10)|(&0xC000<<14)|(&0x300000<<18)|(&0xC0000000<<22)|(&0xC0<<24)],
            27 => [(&0xFCF0F0F3<<0)|(&0xC<<6)|(&0x300<<8)|(&0xC00<<14)|(&0x30000<<18)|(&0xC0000<<24)|(&0x3000000<<26)],
            28 => [(&0xFF00FF<<0)|(&0x30003000<<2)|(&0x3000300<<4)|(&0xC000C000<<28)|(&0xC000C00<<30)],
            29 => [(&0xFF03FFC0<<0)|(&0xC<<2)|(&0xC00000<<12)|(&0xC0000<<14)|(&0x30<<18)|(&0x3<<20)|(&0x300000<<30)],
            30 => [(&0xC30FFFF<<0)|(&0xC30000<<6)|(&0xC0000<<12)|(&0x30000000<<20)|(&0xC3000000<<26)],
            31 => [(&0xFFFFC003<<0)|(&0xC<<2)|(&0x30<<4)|(&0xC0<<6)|(&0x300<<26)|(&0xC00<<28)|(&0x3000<<30)],
            32 => [(&0x3F3333F3<<0)|(&0xC000<<8)|(&0xC0000C<<12)|(&0xC0000C00<<20)|(&0xC0000<<24)],
            33 => [(&0xFCCCCCCF<<0)|(&0x30<<4)|(&0x300<<8)|(&0x3000<<12)|(&0x30000<<20)|(&0x300000<<24)|(&0x3000000<<28)],
            34 => [(&0xCF0F0F3F<<0)|(&0xC00000<<6)|(&0xC000<<8)|(&0xC0<<14)|(&0x30000000<<18)|(&0x300000<<24)|(&0x3000<<26)],
            35 => [(&0xF0F3FCF0<<0)|(&0xC0000<<6)|(&0x3000000<<8)|(&0xC000000<<14)|(&0x3<<18)|(&0xC<<24)|(&0x300<<26)],
            36 => [(&0xCFF30FF<<0)|(&0x3000000<<6)|(&0xC0000000<<12)|(&0xC00<<14)|(&0x30000000<<18)|(&0x300<<20)|(&0xC000<<26)],
            37 => [(&0xFFC0FF03<<0)|(&0xC0000<<2)|(&0xC0<<12)|(&0xC<<14)|(&0x300000<<18)|(&0x30000<<20)|(&0x30<<30)],
            38 => [(&0xFFFF<<0)|(&0x30C0000<<4)|(&0x300000<<6)|(&0x30000<<14)|(&0xC0000000<<18)|(&0xC000000<<26)|(&0x30C00000<<28)],
            39 => [(&0xFFFF0C30<<0)|(&0x30C<<6)|(&0x3<<12)|(&0xC000<<20)|(&0x30C0<<26)],
            40 => [(&0x33333333<<0)|(&0xC00C00C<<4)|(&0xC00<<12)|(&0xC00000<<20)|(&0xC00C00C0<<28)],
            41 => [(&0xCFCCCCFC<<0)|(&0x3000<<8)|(&0x300003<<12)|(&0x30000300<<20)|(&0x30000<<24)],
            42 => [(&0xF3FCF0F<<0)|(&0xC0<<6)|(&0xC0000000<<8)|(&0xC00000<<14)|(&0x3000<<18)|(&0x30<<24)|(&0x30000000<<26)],
            43 => [(&0xF0F0F0F0<<0)|(&0x30300<<2)|(&0xC000000<<6)|(&0x3000000<<10)|(&0xC<<22)|(&0x3<<26)|(&0xC0C00<<30)],
            44 => [(&0x3FFC0FF<<0)|(&0xC000000<<2)|(&0x30000000<<12)|(&0xC0000000<<14)|(&0x300<<18)|(&0xC00<<20)|(&0x3000<<30)],
            45 => [(&0xFF00FF00<<0)|(&0x3000C<<4)|(&0xC00000<<10)|(&0x30<<14)|(&0xC0000<<18)|(&0x3<<22)|(&0x3000C0<<28)],
            46 => [(&0xFFFF<<0)|(&0xC00000<<2)|(&0xC030000<<4)|(&0xC0000<<10)|(&0x30000000<<22)|(&0xC0300000<<28)|(&0x3000000<<30)],
            47 => [(&0xFFFF0000<<0)|(&0xC0<<2)|(&0xC03<<4)|(&0xC<<10)|(&0x3000<<22)|(&0xC030<<28)|(&0x300<<30)],
            48 => [(&0x33333333<<0)|(&0xC000C00<<4)|(&0xC000C<<8)|(&0xC000C000<<24)|(&0xC000C0<<28)],
            49 => [(&0xCCFCCFCC<<0)|(&0x30000000<<8)|(&0x30030<<12)|(&0x3003000<<20)|(&0x3<<24)],
            50 => [(&0xF0F0F0F<<0)|(&0x30000030<<2)|(&0xC000<<6)|(&0x3000<<10)|(&0xC00000<<22)|(&0x300000<<26)|(&0xC00000C0<<30)],
            51 => [(&0xF0F0F0F0<<0)|(&0x3000003<<2)|(&0xC00<<6)|(&0x300<<10)|(&0xC0000<<22)|(&0x30000<<26)|(&0xC00000C<<30)],
            52 => [(&0xC0FF03FF<<0)|(&0xC00<<2)|(&0x3000<<12)|(&0xC000<<14)|(&0x3000000<<18)|(&0xC000000<<20)|(&0x30000000<<30)],
            53 => [(&0xFF00FF00<<0)|(&0xC0003<<4)|(&0xC0<<10)|(&0x300000<<14)|(&0xC<<18)|(&0x30000<<22)|(&0xC00030<<28)],
            54 => [(&0xFFFF<<0)|(&0xCC0000<<6)|(&0x330000<<10)|(&0xCC000000<<22)|(&0x33000000<<26)],
            55 => [(&0xFFFF0000<<0)|(&0xF00<<4)|(&0xF<<8)|(&0xF000<<24)|(&0xF0<<28)],
            56 => [(&0x3F3333F3<<0)|(&0xC00<<8)|(&0xC00C0000<<12)|(&0xC00C<<20)|(&0xC00000<<24)],
            57 => [(&0xCCCCCCCC<<0)|(&0x30030300<<4)|(&0x3000000<<12)|(&0x30<<20)|(&0x303003<<28)],
            58 => [(&0xF0F0F0F<<0)|(&0x30003000<<10)|(&0xC030C030<<16)|(&0xC000C0<<22)],
            59 => [(&0xF0F0F0F0<<0)|(&0xC000C00<<6)|(&0x30C030C<<16)|(&0x30003<<26)],
            60 => [(&0xFF00FF<<0)|(&0xC000300<<4)|(&0xC000<<10)|(&0x30000000<<14)|(&0xC00<<18)|(&0x3000000<<22)|(&0xC0003000<<28)],
            61 => [(&0xFF00FF00<<0)|(&0xC000C0<<10)|(&0x3C003C<<16)|(&0x30003<<22)],
            62 => [(&0xFFFF<<0)|(&0x33000000<<2)|(&0x330000<<8)|(&0xCC000000<<24)|(&0xCC0000<<30)],
            63 => [(&0xFFFF0C30<<0)|(&0xC3<<6)|(&0xC<<12)|(&0x3000<<20)|(&0xC300<<26)],
            64 => [(&0x33F33F33<<0)|(&0xC000000<<8)|(&0xC00C<<12)|(&0xC00C0000<<20)|(&0xC0<<24)],
            65 => [(&0xCCCCCCCC<<0)|(&0x3000300<<12)|(&0x30033003<<16)|(&0x300030<<20)],
            66 => [(&0xF0F0F0F<<0)|(&0x3030<<2)|(&0x3030C0C0<<16)|(&0xC0C00000<<30)],
            67 => [(&0xF0F0F0F0<<0)|(&0x3000300<<10)|(&0xC030C03<<16)|(&0xC000C<<22)],
            68 => [(&0xFF00FF<<0)|(&0x30003000<<14)|(&0xC300C300<<16)|(&0xC000C00<<18)],
            69 => [(&0xFF00FF00<<0)|(&0x300030<<14)|(&0xC300C3<<16)|(&0xC000C<<18)],
            70 => [(&0xFFFF<<0)|(&0x30300000<<2)|(&0x3030000<<4)|(&0xC0C00000<<28)|(&0xC0C0000<<30)],
            71 => [(&0xFFFF03C0<<0)|(&0xC<<2)|(&0x30<<10)|(&0x3<<12)|(&0xC000<<20)|(&0xC00<<22)|(&0x3000<<30)],
            72 => [(&0x33333333<<0)|(&0xC00C0C00<<4)|(&0xC000000<<12)|(&0xC0<<20)|(&0xC0C00C<<28)],
            73 => [(&0xCCCCCCCC<<0)|(&0x30003000<<4)|(&0x3300330<<16)|(&0x30003<<28)],
            74 => [(&0xF0F0F0F<<0)|(&0xF0<<8)|(&0xF0F000<<16)|(&0xF0000000<<24)],
            75 => [(&0xF0F0F0F0<<0)|(&0xF000F00<<8)|(&0xF000F<<24)],
            76 => [(&0xFF00FF<<0)|(&0xF000000<<4)|(&0xF0000F00<<16)|(&0xF000<<28)],
            77 => [(&0xFF00FF00<<0)|(&0xCC00CC<<14)|(&0x330033<<18)],
            78 => [(&0xFFFF<<0)|(&0x330000<<2)|(&0xCC0000<<8)|(&0x33000000<<24)|(&0xCC000000<<30)],
            79 => [(&0xFFFF0000<<0)|(&0x30C<<4)|(&0x30<<6)|(&0x3<<14)|(&0xC000<<18)|(&0xC00<<26)|(&0x30C0<<28)],
            80 => [(&0x33333333<<0)|(&0xC000C000<<4)|(&0xCC00CC0<<16)|(&0xC000C<<28)],
            81 => [(&0xCCCCCCCC<<0)|(&0x33003300<<8)|(&0x330033<<24)],
            82 => [(&0xF3FCF0F<<0)|(&0xC00000<<6)|(&0x30000000<<8)|(&0xC0000000<<14)|(&0x30<<18)|(&0xC0<<24)|(&0x3000<<26)],
            83 => [(&0xF0F0F0F0<<0)|(&0xC000C<<6)|(&0x30003<<10)|(&0xC000C00<<22)|(&0x3000300<<26)],
            84 => [(&0x3FFC0FF<<0)|(&0xC00<<2)|(&0xC0000000<<12)|(&0xC000000<<14)|(&0x3000<<18)|(&0x300<<20)|(&0x30000000<<30)],
            85 => [(&0xFF00FF00<<0)|(&0xC000C<<2)|(&0x30003<<6)|(&0xC000C0<<26)|(&0x300030<<30)],
            86 => [(&0xC30FFFF<<0)|(&0x30C0000<<6)|(&0x30000<<12)|(&0xC0000000<<20)|(&0x30C00000<<26)],
            87 => [(&0xFFFF0000<<0)|(&0x330<<2)|(&0xC<<10)|(&0x3<<14)|(&0xC000<<18)|(&0x3000<<22)|(&0xCC0<<30)],
            88 => [(&0x33333333<<0)|(&0xCC00CC00<<8)|(&0xCC00CC<<24)],
            89 => [(&0xCCCCCCCC<<0)|(&0x300030<<4)|(&0x30003<<12)|(&0x30003000<<20)|(&0x3000300<<28)],
            90 => [(&0xFCF3F0F<<0)|(&0xC0000000<<8)|(&0x300000<<10)|(&0xC0<<14)|(&0x30000000<<18)|(&0xC000<<22)|(&0x30<<24)],
            91 => [(&0xF0F0F0F0<<0)|(&0xF0000<<8)|(&0xF00000F<<16)|(&0xF00<<24)],
            92 => [(&0xCFF30FF<<0)|(&0x300<<6)|(&0x30000000<<12)|(&0xC000<<14)|(&0x3000000<<18)|(&0xC00<<20)|(&0xC0000000<<26)],
            93 => [(&0xFF00FF00<<0)|(&0x330000<<2)|(&0xCC0033<<16)|(&0xCC<<30)],
            94 => [(&0x3C0FFFF<<0)|(&0xC000000<<2)|(&0x30000<<10)|(&0xC0000<<12)|(&0x30000000<<20)|(&0xC0000000<<22)|(&0x300000<<30)],
            95 => [(&0xFFFF0000<<0)|(&0xCC<<6)|(&0x33<<10)|(&0xCC00<<22)|(&0x3300<<26)],
            96 => [(&0x33333333<<0)|(&0xC0C0000<<4)|(&0xC0C00C0C<<16)|(&0xC0C0<<28)],
            97 => [(&0xCCCCCCCC<<0)|(&0x330000<<8)|(&0x33000033<<16)|(&0x3300<<24)],
            98 => [(&0xF0F0F0F<<0)|(&0x303000<<2)|(&0xC0000000<<6)|(&0x30000000<<10)|(&0xC0<<22)|(&0x30<<26)|(&0xC0C000<<30)],
            99 => [(&0xF0F0F0F0<<0)|(&0xF<<8)|(&0xF0F00<<16)|(&0xF000000<<24)],
            100 => [(&0xFF00FF<<0)|(&0x3000C00<<4)|(&0xC0000000<<10)|(&0x3000<<14)|(&0xC000000<<18)|(&0x300<<22)|(&0x3000C000<<28)],
            101 => [(&0xFF00FF00<<0)|(&0x33<<2)|(&0x3300CC<<16)|(&0xCC0000<<30)],
            102 => [(&0xC003FFFF<<0)|(&0xC0000<<2)|(&0x300000<<4)|(&0xC00000<<6)|(&0x3000000<<26)|(&0xC000000<<28)|(&0x30000000<<30)],
            103 => [(&0xFFFF0000<<0)|(&0x303<<2)|(&0xC0C<<4)|(&0x3030<<28)|(&0xC0C0<<30)],
            104 => [(&0x33333333<<0)|(&0xCC<<8)|(&0xCCCC00<<16)|(&0xCC000000<<24)],
            105 => [(&0xCCCCCCCC<<0)|(&0x30003<<4)|(&0x300030<<8)|(&0x3000300<<24)|(&0x30003000<<28)],
            106 => [(&0xF0F0F0F<<0)|(&0xC000C000<<6)|(&0x30C030C0<<16)|(&0x300030<<26)],
            107 => [(&0xF0F0F0F0<<0)|(&0x3000300<<2)|(&0x30003<<8)|(&0xC000C00<<24)|(&0xC000C<<30)],
            108 => [(&0xFF00FF<<0)|(&0xC000C000<<10)|(&0x3C003C00<<16)|(&0x3000300<<22)],
            109 => [(&0xFF00FF00<<0)|(&0x30003<<2)|(&0xC000C<<4)|(&0x300030<<28)|(&0xC000C0<<30)],
            110 => [(&0xFFFF<<0)|(&0x3300000<<2)|(&0xC0000<<10)|(&0x30000<<14)|(&0xC0000000<<18)|(&0x30000000<<22)|(&0xCC00000<<30)],
            111 => [(&0xFFFF0000<<0)|(&0x3300<<2)|(&0x33<<8)|(&0xCC00<<24)|(&0xCC<<30)],
            112 => [(&0x333FF333<<0)|(&0xC00000<<4)|(&0xC000000<<8)|(&0xC0000000<<12)|(&0xC<<20)|(&0xC0<<24)|(&0xC00<<28)],
            113 => [(&0xCCCCCCCC<<0)|(&0x3000300<<4)|(&0x30003<<8)|(&0x30003000<<24)|(&0x300030<<28)],
            114 => [(&0xF0F0F0F<<0)|(&0xC000<<6)|(&0xC00030<<8)|(&0x30000000<<10)|(&0xC0<<22)|(&0xC0003000<<24)|(&0x300000<<26)],
            115 => [(&0xF0F0F0F0<<0)|(&0x30003<<2)|(&0xC000C<<8)|(&0x3000300<<24)|(&0xC000C00<<30)],
            116 => [(&0xFF00FF<<0)|(&0x3003000<<2)|(&0xC0000000<<10)|(&0x30000000<<14)|(&0xC00<<18)|(&0x300<<22)|(&0xC00C000<<30)],
            117 => [(&0xFF00FF00<<0)|(&0x300030<<2)|(&0x30003<<4)|(&0xC000C0<<28)|(&0xC000C<<30)],
            118 => [(&0xFFFF<<0)|(&0x30C30000<<2)|(&0x300000<<6)|(&0xC000000<<26)|(&0xC30C0000<<30)],
            119 => [(&0xFFFF300C<<0)|(&0xC0<<4)|(&0x3<<6)|(&0x30<<10)|(&0xC00<<22)|(&0xC000<<26)|(&0x300<<28)],
            120 => [(&0x33333333<<0)|(&0xC000C00<<12)|(&0xC00CC00C<<16)|(&0xC000C0<<20)],
            121 => [(&0xCCCFFCCC<<0)|(&0x30<<4)|(&0x30000000<<8)|(&0x300000<<12)|(&0x300<<20)|(&0x3<<24)|(&0x3000000<<28)],
            122 => [(&0xF0F0F0F<<0)|(&0xF000F000<<8)|(&0xF000F0<<24)],
            123 => [(&0xF0FCF3F0<<0)|(&0x3000000<<8)|(&0x3<<10)|(&0xC00<<14)|(&0x30000<<18)|(&0xC000000<<22)|(&0xC<<24)],
            124 => [(&0xFF00FF<<0)|(&0xCC00CC00<<14)|(&0x33003300<<18)],
            125 => [(&0xFF0CFF30<<0)|(&0x30000<<6)|(&0xC00000<<12)|(&0xC<<14)|(&0x300000<<18)|(&0x3<<20)|(&0xC0<<26)],
            126 => [(&0xFFFF<<0)|(&0x300000<<6)|(&0xC30000<<8)|(&0xC0000<<10)|(&0x30000000<<22)|(&0xC3000000<<24)|(&0xC000000<<26)],
            127 => [(&0xFFFF0000<<0)|(&0x30C3<<2)|(&0x30<<6)|(&0xC00<<26)|(&0xC30C<<30)],
            128 => [(&0x33333333<<0)|(&0xC0000000<<4)|(&0xC00C0<<8)|(&0xC00<<12)|(&0xC00000<<20)|(&0xC00C000<<24)|(&0xC<<28)],
            129 => [(&0xCCFCCFCC<<0)|(&0x3000000<<8)|(&0x3003<<12)|(&0x30030000<<20)|(&0x30<<24)],
            130 => [(&0xF0F0F0F<<0)|(&0xC000C0<<6)|(&0x300030<<10)|(&0xC000C000<<22)|(&0x30003000<<26)],
            131 => [(&0xF0F3FCF0<<0)|(&0xC<<6)|(&0xC000000<<8)|(&0xC0000<<14)|(&0x300<<18)|(&0x3<<24)|(&0x3000000<<26)],
            132 => [(&0xFF00FF<<0)|(&0xC000C00<<2)|(&0x3000300<<6)|(&0xC000C000<<26)|(&0x30003000<<30)],
            133 => [(&0xFFC0FF03<<0)|(&0xC<<2)|(&0x30<<12)|(&0xC0<<14)|(&0x30000<<18)|(&0xC0000<<20)|(&0x300000<<30)],
            134 => [(&0xFFFF<<0)|(&0xF00000<<4)|(&0xF0000<<12)|(&0xF0000000<<20)|(&0xF000000<<28)],
            135 => [(&0xFFFF0000<<0)|(&0xC0<<2)|(&0x3C<<8)|(&0x3<<14)|(&0xC000<<18)|(&0x3C00<<24)|(&0x300<<30)],
            136 => [(&0x33333333<<0)|(&0xC000C0<<4)|(&0xC000C<<12)|(&0xC000C000<<20)|(&0xC000C00<<28)],
            137 => [(&0xCFCCCCFC<<0)|(&0x300<<8)|(&0x30030000<<12)|(&0x3003<<20)|(&0x300000<<24)],
            138 => [(&0xF0F0F0F<<0)|(&0xF00000<<8)|(&0xF00000F0<<16)|(&0xF000<<24)],
            139 => [(&0xFCF0F0F3<<0)|(&0xC0000<<6)|(&0xC00<<8)|(&0xC<<14)|(&0x3000000<<18)|(&0x30000<<24)|(&0x300<<26)],
            140 => [(&0xFF00FF<<0)|(&0x33000000<<2)|(&0xCC003300<<16)|(&0xCC00<<30)],
            141 => [(&0xFF03FFC0<<0)|(&0xC0000<<2)|(&0x300000<<12)|(&0xC00000<<14)|(&0x3<<18)|(&0xC<<20)|(&0x30<<30)],
            142 => [(&0xFFFF<<0)|(&0xF000000<<4)|(&0xF0000<<8)|(&0xF0000000<<24)|(&0xF00000<<28)],
            143 => [(&0xFFFF0000<<0)|(&0xF0<<4)|(&0xF<<12)|(&0xF000<<20)|(&0xF00<<28)],
            144 => [(&0x33333333<<0)|(&0xC000C<<4)|(&0xC000C0<<8)|(&0xC000C00<<24)|(&0xC000C000<<28)],
            145 => [(&0xCCCCCCCC<<0)|(&0x3000<<4)|(&0x300003<<8)|(&0x3000000<<12)|(&0x30<<20)|(&0x30000300<<24)|(&0x30000<<28)],
            146 => [(&0xF0F0F0F<<0)|(&0x30003000<<2)|(&0x300030<<8)|(&0xC000C000<<24)|(&0xC000C0<<30)],
            147 => [(&0xF0F0F0F0<<0)|(&0xC000000<<6)|(&0x3000C<<8)|(&0x300<<10)|(&0xC0000<<22)|(&0x3000C00<<24)|(&0x3<<26)],
            148 => [(&0xFF00FF<<0)|(&0x3000300<<2)|(&0xC000C00<<4)|(&0x30003000<<28)|(&0xC000C000<<30)],
            149 => [(&0xFF00FF00<<0)|(&0x300003<<2)|(&0xC0<<10)|(&0x30<<14)|(&0xC0000<<18)|(&0x30000<<22)|(&0xC0000C<<30)],
            150 => [(&0xFFFF<<0)|(&0xF0000<<4)|(&0xF00000<<8)|(&0xF000000<<24)|(&0xF0000000<<28)],
            151 => [(&0xFFFF0000<<0)|(&0xF<<4)|(&0xF0<<8)|(&0xF00<<24)|(&0xF000<<28)],
            152 => [(&0xF333333F<<0)|(&0xC00000<<4)|(&0xC000<<8)|(&0xC0<<12)|(&0xC000000<<20)|(&0xC0000<<24)|(&0xC00<<28)],
            153 => [(&0xCCCCCCCC<<0)|(&0x30000000<<4)|(&0x30030<<8)|(&0x300<<12)|(&0x300000<<20)|(&0x3003000<<24)|(&0x3<<28)],
            154 => [(&0x3F0F0FCF<<0)|(&0x3000<<8)|(&0x300000<<10)|(&0xC0000000<<14)|(&0x30<<18)|(&0xC000<<22)|(&0xC00000<<24)],
            155 => [(&0xF0F0F0F0<<0)|(&0xC00<<6)|(&0xC0003<<8)|(&0x3000000<<10)|(&0xC<<22)|(&0xC000300<<24)|(&0x30000<<26)],
            156 => [(&0x30FF0CFF<<0)|(&0x300<<6)|(&0xC000<<12)|(&0xC000000<<14)|(&0x3000<<18)|(&0x3000000<<20)|(&0xC0000000<<26)],
            157 => [(&0xFF00FF00<<0)|(&0x30030<<2)|(&0xC00000<<10)|(&0x300000<<14)|(&0xC<<18)|(&0x3<<22)|(&0xC00C0<<30)],
            158 => [(&0x300CFFFF<<0)|(&0x300000<<4)|(&0x3000000<<6)|(&0x30000<<10)|(&0xC0000000<<22)|(&0xC00000<<26)|(&0xC000000<<28)],
            159 => [(&0xFFFFC003<<0)|(&0xC00<<2)|(&0xC0<<4)|(&0xC<<6)|(&0x3000<<26)|(&0x300<<28)|(&0x30<<30)],
            160 => [(&0x33333333<<0)|(&0xC000<<4)|(&0xC0000C<<8)|(&0xC000000<<12)|(&0xC0<<20)|(&0xC0000C00<<24)|(&0xC0000<<28)],
            161 => [(&0xCCCCCCCC<<0)|(&0x30303030<<12)|(&0x3030303<<20)],
            162 => [(&0xF0F0F0F<<0)|(&0xC0000000<<6)|(&0x3000C0<<8)|(&0x3000<<10)|(&0xC00000<<22)|(&0x3000C000<<24)|(&0x30<<26)],
            163 => [(&0xF0F0F0F0<<0)|(&0xC0C0C0C<<14)|(&0x3030303<<18)],
            164 => [(&0xFF00FF<<0)|(&0x30000300<<2)|(&0xC000<<10)|(&0x3000<<14)|(&0xC000000<<18)|(&0x3000000<<22)|(&0xC0000C00<<30)],
            165 => [(&0xFF00FF00<<0)|(&0xF000F0<<12)|(&0xF000F<<20)],
            166 => [(&0xFFFF<<0)|(&0xC00000<<2)|(&0x3C0000<<8)|(&0x30000<<14)|(&0xC0000000<<18)|(&0x3C000000<<24)|(&0x3000000<<30)],
            167 => [(&0xFFFF0000<<0)|(&0x30<<6)|(&0xC3<<8)|(&0xC<<10)|(&0x3000<<22)|(&0xC300<<24)|(&0xC00<<26)],
            168 => [(&0x33333333<<0)|(&0xC0C0C0C0<<12)|(&0xC0C0C0C<<20)],
            169 => [(&0xCCCCCCCC<<0)|(&0x303<<4)|(&0x3033030<<16)|(&0x30300000<<28)],
            170 => [(&0xF0F0F0F<<0)|(&0xC0C0C0C0<<14)|(&0x30303030<<18)],
            171 => [(&0xF0F0F0F0<<0)|(&0x3030000<<2)|(&0xC0C0303<<16)|(&0xC0C<<30)],
            172 => [(&0xFF00FF<<0)|(&0xF000F000<<12)|(&0xF000F00<<20)],
            173 => [(&0xFF00FF00<<0)|(&0xF<<4)|(&0xF00F0<<16)|(&0xF00000<<28)],
            174 => [(&0xFFFF<<0)|(&0xC0C0000<<2)|(&0x3030000<<6)|(&0xC0C00000<<26)|(&0x30300000<<30)],
            175 => [(&0xFFFF0000<<0)|(&0xC0C<<2)|(&0x303<<6)|(&0xC0C0<<26)|(&0x3030<<30)],
            176 => [(&0x33333333<<0)|(&0xC0C<<4)|(&0xC0CC0C0<<16)|(&0xC0C00000<<28)],
            177 => [(&0xCCCCCCCC<<0)|(&0x3030000<<4)|(&0x30300303<<16)|(&0x3030<<28)],
            178 => [(&0xF0F0F0F<<0)|(&0x30300000<<2)|(&0xC0C03030<<16)|(&0xC0C0<<30)],
            179 => [(&0xF0F0F0F0<<0)|(&0x303<<2)|(&0x3030C0C<<16)|(&0xC0C0000<<30)],
            180 => [(&0xFF00FF<<0)|(&0xF00<<4)|(&0xF00F000<<16)|(&0xF0000000<<28)],
            181 => [(&0xFF00FF00<<0)|(&0xF0000<<4)|(&0xF0000F<<16)|(&0xF0<<28)],
            182 => [(&0xFFFF<<0)|(&0x3030000<<2)|(&0xC0C0000<<4)|(&0x30300000<<28)|(&0xC0C00000<<30)],
            183 => [(&0xFFFF0000<<0)|(&0x3030<<2)|(&0x303<<4)|(&0xC0C0<<28)|(&0xC0C<<30)],
        ]);
        Self { r_p, e_op, c_o }
    }
}
