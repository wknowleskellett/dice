// use character::Character;
use dice::prelude::*;
use dice::{
    roll::Roll,
    utils::{
        self,
        dice::{Die, Disadvantage, ModifiedDie, MultipliedDie, SumDie},
    },
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

    let disadvantage_d20 = Disadvantage::new(d(20), d(20));

    let mut results = disadvantage_d20
        .get_stats()
        .into_iter()
        .collect::<Vec<(i32, f32)>>();
    results.sort_by_key(|k| k.0);
    let total = results.iter().map(|pair| pair.1).sum::<f32>();

    for (a, b) in results.iter() {
        println!("{},{}", a, (b.clone() * 100.0) as f32 / total);
    }
}
