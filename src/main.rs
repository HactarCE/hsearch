#![allow(unused_crate_dependencies)]

fn main() {
    let t = std::time::Instant::now();
    for i in 0..20 {
        let scramble = hsearch::scramble(i);
        // println!("{}", util::twists_to_string(&scramble));
        let sol = hsearch::solve(scramble);
        // println!("{}", util::twists_to_string(&sol));
        println!();
    }
    println!("Done in {:?}", t.elapsed());
}
