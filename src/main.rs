// use character::Character;
use dice::prelude::*;
use dice::{
    roll::Roll,
    utils::dice::{Advantage, ConstRoll, Disadvantage, SumDie},
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
    let d2_d2_five = SumDie::new(vec![Box::new(d(2)), Box::new(d(2)), Box::new(ConstRoll(5))]);

    let mut dice: Vec<(&str, Box<dyn Roll<Output = i32>>)> = vec![
        ("d20", Box::new(d20)),
        ("disadvantage", Box::new(disadvantage_d20)),
        ("advantage", Box::new(advantage_d20)),
        ("d2+d2+5", Box::new(d2_d2_five)),
        ("five", Box::new(ConstRoll(5))),
        ("d6, d6", Box::new(SumDie::new(box_dice!(i32; d(6), d(6))))),
    ];
    for (name, die) in dice.iter_mut() {
        let mut results = die.get_stats().into_iter().collect::<Vec<(i32, f32)>>();
        results.sort_by_key(|k| k.0);

        println!("{name}");

        for (a, b) in results {
            println!("{},{:.2}%", a, b * 100.0);
        }
        println!();
    }

    println!();
    let mut sp = whatever::Specials::new();
    println!("{}", sp.roll());
}

mod whatever {
    use dice::{
        prelude::d,
        roll::compound::CompoundRoll,
        roll_tuple, roll_tuple_pattern, roll_tuple_type,
        utils::dice::{Die, MapDie},
    };
    use rand::rngs::ThreadRng;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Entree {
        Pierogies,
        CaesarSalad,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Soup {
        CreamOfMushroom,
        Borscht,
    }

    pub struct Specials {
        // [roll_tuple_type] is used here to generate the type signature of the [RollTuple] type used
        d: roll_tuple_type!(
            MapDie<Entree, Die<ThreadRng>, Box<dyn Fn(i32) -> Entree>>,
            MapDie<Soup, Die<ThreadRng>, Box<dyn Fn(i32) -> Soup>>
        ),
    }

    impl Specials {
        pub fn new() -> Self {
            let entree_die = MapDie::new(
                Box::new(|n: i32| match n {
                    1 => Entree::CaesarSalad,
                    2 => Entree::Pierogies,
                    _ => panic!("Two sided die rolled something other than 1 or 2"),
                })  as Box<dyn Fn(i32) -> Entree>,
                d(2),
            );
            let soup_die = MapDie::new(
                Box::new(|n: i32| match n {
                    1 => Soup::CreamOfMushroom,
                    2 => Soup::Borscht,
                    _ => panic!("Two sided die rolled something other than 1 or 2"),
                }) as Box<dyn Fn(i32) -> Soup>,
                d(2),
            );
            // [roll_tuple] is used here to fold these two dice into a [RollTuple] type
            Self { d: roll_tuple!(entree_die, soup_die) }
        }
    }

    impl CompoundRoll for Specials {
        type Output = String;

        type DiceSet = roll_tuple_type!(
            MapDie<Entree, Die<ThreadRng>, Box<dyn Fn(i32) -> Entree>>,
            MapDie<Soup, Die<ThreadRng>, Box<dyn Fn(i32) -> Soup>>
        );

        fn get_dice(&self) -> &Self::DiceSet {
            &self.d
        }

        fn get_dice_mut(&mut self) -> &mut Self::DiceSet {
            &mut self.d
        }

        fn calculate(
            &self,
            results: <Self::DiceSet as dice::roll::compound::RollTuple>::Outputs,
        ) -> Self::Output {
            // [roll_tuple_pattern] is useful for unpacking the dice results.
            let roll_tuple_pattern!(entree, soup) = results;
            format!(
                "Tonight's specials are an entree of {:?}, and the soup du jour is {:?}.",
                entree, soup
            )
        }
    }
}
