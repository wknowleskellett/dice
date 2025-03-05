use character::Character;
use dice::{roll::Roll, utils::{dice::Die, HighestDice, RngOven}};
use rand::thread_rng;

mod character {
    use dice::roll::Roll;
    use dice::utils::HighestDice;
    use dice::utils::{dice::Die,RngOven};
    use rand::prelude::*;
    use std::ops::Index;
    use std::ops::IndexMut;

    #[derive(Debug)]
    pub struct Character {
        st: i32,
        dex: i32,
        con: i32,
        int: i32,
        wis: i32,
        cha: i32,
    }

    impl Character {
        pub fn _new(
            st: i32,
            dex: i32,
            con: i32,
            int: i32,
            wis: i32,
            cha: i32,
            ) -> Self {
            Self {
                st,
                dex,
                con,
                int,
                wis,
                cha
            }
        }


        pub fn generate() -> Self {
            let mut die = HighestDice::new((1..=4).map(|_| Box::new(
                Die::new(6).bake(thread_rng())
            ) as Box<dyn Roll>).collect(), 3);
            let mut rng = thread_rng();
            let mut retval = Self {
                st: 0,
                dex: 0,
                con: 0,
                int: 0,
                wis: 0,
                cha: 0,
            };
            let mut scores = vec![0; 6]
                .iter()
                .map(|_| die.roll())
                .collect::<Vec<_>>();
            scores.shuffle(&mut rng);
            scores.iter()
                .enumerate()
                .for_each(|(i, n)| retval[i] = *n);
            retval
        }
    }

    impl Index<usize> for Character {
        type Output = i32;

        fn index<'a>(&'a self, i: usize) -> &'a Self::Output {
            match i {
                0 => &self.st,
                1 => &self.dex,
                2 => &self.con,
                3 => &self.int,
                4 => &self.wis,
                5 => &self.cha,
                _ => panic!("Index out of bounds (0-5)"),
            }
        }
    }

    impl IndexMut<usize> for Character {

        fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut i32 {
            match i {
                0 => &mut self.st,
                1 => &mut self.dex,
                2 => &mut self.con,
                3 => &mut self.int,
                4 => &mut self.wis,
                5 => &mut self.cha,
                _ => panic!("Index out of bounds (0-5)"),
            }
        }
    }

}

fn main() {
    // let _d2 = Die::new(2);
    // let _d4 = Die::new(4);
    // let _d6 = Die::new(6);
    // let _d8 = Die::new(8);
    // let _d10 = Die::new(10);
    // let _d100 = Die::new(100);
    // let _d12 = Die::new(12);
    // let _d20 = Die::new(20);
    // let mut my_guy = character::Character::generate();
    // println!("{:?}", my_guy);
    // for i in 0..6u32 {
    //     my_guy[usize::try_from(i).unwrap()] = i32::try_from(i).unwrap();
    // }
    // println!("{:?}", my_guy);

    // println!("{:?}", Character::generate());
    let _: Vec<_> = (1..=10).map(|_| println!("{:?}", Character::generate())).collect();

    let mut die = HighestDice::new((1..=4).map(|_| Box::new(
        Die::new(6).bake(thread_rng())
    ) as Box<dyn Roll>).collect(), 3);
    let mut counts = [0; 19];
    for _ in 0..1000000 {
        counts[TryInto::<usize>::try_into(die.roll()).unwrap()] += 1;
    }
    println!("{:?}", counts)
    
}
