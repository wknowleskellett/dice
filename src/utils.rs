pub mod dice {
    use std::{collections::HashMap, hash::Hash};

    use rand::{rngs::ThreadRng, thread_rng, Rng};

    use crate::roll::{CompoundRoll, Roll, RollTuple};

    impl Roll for i32 {
        type Output = i32;

        fn roll(&mut self) -> Self::Output {
            *self
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            HashMap::from([(*self, 1.0)])
        }
    }

    #[derive(Debug)]
    pub struct Die<R: Rng> {
        d: i32,
        rng: R,
    }

    impl Die<ThreadRng> {
        pub fn new_baked(d: i32) -> Self {
            assert!(d > 0);
            Self {
                d,
                rng: thread_rng(),
            }
        }
    }

    impl<R: Rng> Die<R> {
        pub fn new(d: i32, rng: R) -> Self {
            assert!(d > 0);
            Self { d, rng }
        }
    }

    impl<R: Rng> Roll for Die<R> {
        type Output = i32;

        fn roll(&mut self) -> Self::Output {
            self.rng.gen_range(1..=self.d)
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            let mut stats = HashMap::new();
            let frac = 1.0 / (self.d as f32);
            for n in 1..=self.d {
                stats.insert(n, frac);
            }

            stats
        }
    }

    #[derive(Debug)]
    pub struct ModifiedDie<T: Roll> {
        d: T,
        m: i32,
    }

    impl<T: Roll> ModifiedDie<T> {
        pub fn new(d: T, m: i32) -> Self {
            Self { d, m }
        }
    }

    impl<T: Roll<Output = i32>> Roll for ModifiedDie<T> {
        type Output = i32;

        fn roll(&mut self) -> Self::Output {
            self.d.roll() + self.m
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            let mut stats = HashMap::new();

            let child_stats = self.d.get_stats();

            for (result, count) in child_stats.into_iter() {
                stats.insert(result + self.m, count);
            }

            stats
        }
    }

    #[derive(Debug)]
    pub struct MultipliedDie<T: Roll> {
        d: T,
        m: i32,
    }

    impl<T: Roll> MultipliedDie<T> {
        pub fn new(d: T, m: i32) -> Self {
            Self { d, m }
        }
    }

    impl<T: Roll<Output = i32>> Roll for MultipliedDie<T> {
        type Output = i32;

        fn roll(&mut self) -> i32 {
            self.d.roll() * self.m
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            if self.m == 0 {
                // In case the multiplier is 0, the die always rolls 0
                return HashMap::from([(0, 1.0)]);
            }

            let mut stats = HashMap::new();

            let child_stats = self.d.get_stats();
            for (result, count) in child_stats.into_iter() {
                stats.insert(result * self.m, count);
            }

            stats
        }
    }

    // #[derive(Debug)]
    pub struct SumDie {
        dice: Vec<Box<dyn Roll<Output = i32>>>,
    }

    impl SumDie {
        pub fn new(dice: Vec<Box<dyn Roll<Output = i32>>>) -> Self {
            Self { dice }
        }
    }

    impl Roll for SumDie {
        type Output = i32;

        fn roll(&mut self) -> i32 {
            self.dice.iter_mut().map(|die| die.roll()).sum()
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            let mut cumulative_stats = HashMap::from([(0, 1.0)]);
            for d in self.dice.iter() {
                let old_stats = cumulative_stats;
                let stats = d.get_stats();

                cumulative_stats = HashMap::new();
                for (result, count_n) in stats.into_iter() {
                    for (cumulative_result, count) in old_stats.iter() {
                        let stat = cumulative_stats
                            .entry(result + cumulative_result)
                            .or_insert(0.0);
                        *stat += count_n * count;
                    }
                }
            }
            cumulative_stats
        }
    }

    pub struct Advantage<T: Roll<Output = i32>, U: Roll<Output = i32>> {
        d_1: T,
        d_2: U,
    }

    impl<T: Roll<Output = i32>, U: Roll<Output = i32>> Advantage<T, U> {
        pub fn new(d_1: T, d_2: U) -> Advantage<T, U> {
            Advantage { d_1, d_2 }
        }
    }

    impl<T: Roll<Output = i32>, U: Roll<Output = i32>> Roll for Advantage<T, U> {
        type Output = i32;

        fn roll(&mut self) -> i32 {
            std::cmp::max(self.d_1.roll(), self.d_2.roll())
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            let mut stats = HashMap::new();
            let stats_1 = self.d_1.get_stats();
            let stats_2 = self.d_2.get_stats();
            for (side_1, count_1) in stats_1.iter() {
                for (side_2, count_2) in stats_2.iter() {
                    let result = std::cmp::max(side_1, side_2).clone();
                    let count = stats.entry(result).or_insert(0.0);
                    *count += count_1 * count_2;
                }
            }

            stats
        }
    }

    pub struct Disadvantage<T: Roll<Output = i32>, U: Roll<Output = i32>> {
        d_1: T,
        d_2: U,
    }

    impl<T: Roll<Output = i32>, U: Roll<Output = i32>> Disadvantage<T, U> {
        pub fn new(d_1: T, d_2: U) -> Disadvantage<T, U> {
            Disadvantage { d_1, d_2 }
        }
    }

    impl<T: Roll<Output = i32>, U: Roll<Output = i32>> Roll for Disadvantage<T, U> {
        type Output = i32;

        fn roll(&mut self) -> i32 {
            std::cmp::min(self.d_1.roll(), self.d_2.roll())
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            let mut stats = HashMap::new();
            let stats_1 = self.d_1.get_stats();
            let stats_2 = self.d_2.get_stats();
            for (side_1, count_1) in stats_1.iter() {
                for (side_2, count_2) in stats_2.iter() {
                    let result = std::cmp::min(side_1, side_2).clone();
                    let count = stats.entry(result).or_insert(0.0);
                    *count += count_1 * count_2;
                }
            }

            stats
        }
    }

    // pub struct FnTest<M: Fn(i32) -> T, T> {
    //     my_map: M,
    // }

    // impl <M: Fn(i32) -> T, T> FnTest<M, T> {
    //     fn do_it(&self, n: i32) -> T {
    //         self.my_map(n)
    //     }
    // }

    // pub struct MapDie<M: Fn(i32) -> T, T, R: Roll> {
    //     die_map: M,
    //     die: R,
    // }

    // impl<M: Fn(i32) -> T, T, R: Roll> MapDie<M, T, R> {
    //     pub fn new(die_map: M, d: R) -> Self {
    //         Self { die_map, die: d }
    //     }
    // }

    // impl<M, T, R> CompoundRoll for MapDie<M, T, R>
    // where
    //     M: Fn(i32) -> T,
    //     R: Roll,
    //     R::Output: Eq + Hash + Clone,
    // {
    //     type Output = T;

    //     type DiceSet = (R, ());

    //     // fn get_dice(&self) -> &Self::DiceSet {
    //     //     (self.die, ())
    //     // }

    //     // fn get_dice_mut(&mut self) -> &mut Self::DiceSet {
    //     //     &mut self.die
    //     // }

    //     fn calculate(&self, results: <Self::DiceSet as RollTuple>::Outputs) -> Self::Output {
    //         self.die_map(0)
    //     }

    //     fn get_stats_all(&self) -> HashMap<<Self::DiceSet as RollTuple>::Outputs, f32> {
    //         todo!()
    //     }

    //     fn roll_all(&mut self) -> <Self::DiceSet as RollTuple>::Outputs {
    //         todo!()
    //     }
    // }

    // pub struct ConcatenateDie<T: RollTuple> {
    //     dice: T,
    // }

    // impl<T: RollTuple> CompoundRoll for ConcatenateDie<T> {
    //     type Output = String;
    
    //     type DiceSet = ();
    
    //     fn get_stats_all(&self) -> HashMap<<Self::DiceSet as RollTuple>::Outputs, f32> {
    //         todo!()
    //     }
    
    //     fn roll_all(&mut self) -> <Self::DiceSet as RollTuple>::Outputs {
    //         todo!()
    //     }
    
    //     fn calculate(&self, results: <Self::DiceSet as RollTuple>::Outputs) -> Self::Output {
    //         todo!()
    //     }
    // }
}

// // #[derive(Debug)]
// pub struct HighestDice {
//     dice: Vec<Box<dyn Roll>>,
//     count: usize,
// }

// impl HighestDice {
//     pub fn new(dice: Vec<Box<dyn Roll>>, count: usize) -> Self {
//         assert!(count <= dice.len());
//         Self {
//             dice,
//             count,
//         }
//     }
// }

// impl Roll for HighestDice {
//     fn roll(&mut self) -> i32 {
//         let mut results = self.dice.iter_mut().map(|die_box| (*die_box).roll())
//         .collect::<Vec<i32>>();
//         results.sort_by_key(|w| Reverse(*w));
//         results.truncate(self.count);
//         results.iter().sum()
//     }
// }
