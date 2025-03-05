use std::cmp::Reverse;

use rand::Rng;

use crate::roll::{CriticalFailure, CriticalSuccess, Explosion, Roll, RollRng};

pub mod dice {
    use rand::Rng;

    use crate::roll::{CriticalFailure, CriticalSuccess, RollRng};

    #[derive(Debug)]
    pub struct Die {
        d: i32,
    }
    
    impl Die {
        pub fn new(d: u16) -> Self {
            Self {
                d: d.into(),
            }
        }
    }
    
    impl RollRng for Die {
        fn roll_rng<U: Rng>(&self, rng: &mut U) -> i32 {
            rng.gen_range(1..=self.d)
        }
    }
    
    impl CriticalFailure for Die {
        fn is_critical_failure(&self, n: i32) -> bool {
            n == 1
        }
    }
    
    impl CriticalSuccess for Die {
        fn is_critical_success(&self, n: i32) -> bool {
            n == self.d
        }
    }
}

pub struct BakedRng<T: RollRng, U: Rng> {
    d: T,
    rng: U,
}

pub trait RngOven: RollRng + Sized {
    fn bake<'a, T: Rng>(self: Self, rng: T) -> BakedRng<Self, T> {
        BakedRng::new(self, rng)
    }
}

impl <T: RollRng> RngOven for T {}

impl <T: RollRng, U: Rng> BakedRng<T, U> {
    pub fn new(d: T, rng: U) -> Self {
        Self {
            d,
            rng,
        }
    }
}

impl <'a, T: RollRng, U: Rng> Roll for BakedRng<T, U> {
    fn roll(&mut self) -> i32 {
        self.d.roll_rng(&mut self.rng)
    }
}

impl <'a, T: RollRng + CriticalSuccess, U: Rng> CriticalSuccess for BakedRng<T, U> {
    fn is_critical_success(&self, n: i32) -> bool {
        self.d.is_critical_success(n)
    }
}

impl <'a, T: RollRng + CriticalFailure, U: Rng> CriticalFailure for BakedRng<T, U> {
    fn is_critical_failure(&self, n: i32) -> bool {
        self.d.is_critical_failure(n)
    }
}

impl <'a, T: RollRng + Explosion, U: Rng> Explosion for BakedRng<T, U> {    
    fn explodes(&self, n: i32) -> bool {
        self.d.explodes(n)
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
pub struct AddedDice {
    dice: Vec<Box<dyn Roll>>,
}

impl AddedDice {
    pub fn new(dice: Vec<Box<dyn Roll>>) -> Self {
        Self {
            dice
        }
    }
}

impl Roll for AddedDice {
    fn roll(&mut self) -> i32 {
        self.dice.iter_mut().map(|die_box| (*die_box).roll()).sum()
    }
}

#[derive(Debug)]
pub struct RepeatDie<T: Roll> {
    n: i32,
    d: T,
}

impl<T: Roll> RepeatDie<T> {
    pub fn new(n: i32, d: T) -> Self {
        Self {
            n,
            d,
        }
    }
}

impl<T: Roll> Roll for RepeatDie<T> {
    fn roll(&mut self) -> i32 {
        self.d.repeat(self.n)
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