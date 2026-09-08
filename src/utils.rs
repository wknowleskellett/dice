use std::cmp::Reverse;

use crate::roll::Roll;

pub mod dice {
    use rand::{Rng, rngs::ThreadRng, thread_rng};

    use crate::roll::{Roll};

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
    
    impl <R: Rng> Die<R> {

        pub fn new(d: i32, rng: R) -> Self {
            assert!(d > 0);
            Self {
                d,
                rng
            }
        }
    }
    
    impl <R: Rng> Roll for Die<R> {
        fn roll(&mut self) -> i32 {
            self.rng.gen_range(1..=self.d)
        }
    }
}



#[derive(Debug)]
pub struct ModifiedDie<T: Roll> {
    d: T,
    m: i32,
}

impl <T: Roll> ModifiedDie<T> {
    pub fn new(d: T, m: i32) -> Self {
        Self {
            d,
            m,
        }
    }
}

impl<T: Roll> Roll for ModifiedDie<T> {
    fn roll(&mut self) -> i32 {
        self.d.roll() + self.m
    }
}

#[derive(Debug)]
pub struct MultipliedDie<T: Roll> {
    d: T,
    m: i32,
}

impl <T: Roll> MultipliedDie<T> {
    pub fn new(d: T, m: i32) -> Self {
        Self {
            d,
            m,
        }
    }
}

impl <T: Roll> Roll for MultipliedDie<T> {
    fn roll(&mut self) -> i32 {
        self.d.roll() * self.m
    }
}


// #[derive(Debug)]
pub struct HighestDice {
    dice: Vec<Box<dyn Roll>>,
    count: usize,
}

impl HighestDice {
    pub fn new(dice: Vec<Box<dyn Roll>>, count: usize) -> Self {
        assert!(count <= dice.len());
        Self {
            dice,
            count,
        }
    }
}

impl Roll for HighestDice {
    fn roll(&mut self) -> i32 {
        let mut results = self.dice.iter_mut().map(|die_box| (*die_box).roll())
        .collect::<Vec<i32>>();
        results.sort_by_key(|w| Reverse(*w));
        results.truncate(self.count);
        results.iter().sum()
    }
}