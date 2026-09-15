macro_rules! apply_permutation_lut {
    ($int_type:ty, $input:expr, $twist:expr, [$($i:literal => [ $((& $mask:literal << $delta:literal))|* ]),* $(,)?]) => {
        {
            let input: $int_type = $input;
            match Twist::to_index($twist) {
                $( $i => $( <$int_type>::rotate_left(input & $mask, $delta) )|* , )*
                _ => panic!("twist not allowed in this stage"),
            }
        }
    };
}

pub fn update_orientations_u64(input: u64, [m1, ma, mb]: [u64; 3]) -> u64 {
    let a = input;
    // Swap odd & even bits
    let b = ((input >> 1) & 0x5555_5555_5555_5555) | ((input & 0x5555_5555_5555_5555) << 1);
    // Apply masks
    m1 ^ (ma & a) ^ (mb & b)
}

pub fn update_orientations_u32(input: u32, [m1, ma, mb]: [u32; 3]) -> u32 {
    let a = input;
    // Swap odd & even bits
    let b = ((input >> 1) & 0x5555_5555) | ((input & 0x5555_5555) << 1);
    // Apply masks
    m1 ^ (ma & a) ^ (mb & b)
}
