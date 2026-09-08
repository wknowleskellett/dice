// use character::Character;
use dice::{roll::Roll, utils::{dice::Die}};

fn main() {
    let mut d6 = Die::new_baked(6);
    println!("{} was rolled", d6.roll());

}
