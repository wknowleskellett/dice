// use character::Character;
use dice::prelude::*;
use dice::{
    roll::Roll,
    utils::dice::{Advantage, Disadvantage, SumDie},
};

fn main() {
    // let mut d6 = Die::new_baked(6);
    // println!("{} was rolled", d6.roll());

    // let combined = SumDie::new(vec![
    //     Box::new(ModifiedDie::new(d(10), -1)),
    //     Box::new(MultipliedDie::new(ModifiedDie::new(d(10), -1), 10)),
    //     ]);
    // let mut results = combined.get_stats().into_iter().collect::<Vec<(i32, i32)>>();
    // results.sort_by_key(|k| k.0);
    // println!("stats are {:?}", results);

    let d20 = d(20);
    let disadvantage_d20 = Disadvantage::new(d(20), d(20));
    let advantage_d20 = Advantage::new(d(20), d(20));
    let d4d6 = SumDie::new(vec![Box::new(d(4)), Box::new(d(6))]);

    let mut dice: Vec<Box<dyn Roll<Output = i32>>> = vec![
        Box::new(d20),
        Box::new(disadvantage_d20),
        Box::new(advantage_d20),
        Box::new(d4d6),
    ];
    for die in dice.iter_mut() {
        let mut results = die.get_stats().into_iter().collect::<Vec<(i32, f32)>>();
        results.sort_by_key(|k| k.0);

        for (a, b) in results.iter() {
            println!("{},{}", a, b.clone() * 100.0);
        }
    }
}
