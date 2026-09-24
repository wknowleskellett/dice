pub mod dice {
    use std::{
        collections::HashMap,
        hash::Hash,
        iter::{Product, Sum},
        marker::PhantomData,
        ops::{Add, Mul},
    };

    use rand::{RngExt, rng, rngs::ThreadRng};

    use crate::{prelude::*, roll::compound::*};

    #[derive(Debug)]
    pub struct ConstRoll<T>(pub T);

    impl<T> ConstRoll<T>
    where
        T: Eq + Hash + Clone,
    {
        pub fn new(constant: T) -> Self {
            Self(constant)
        }
    }

    impl<T> Roll for ConstRoll<T>
    where
        T: Eq + Hash + Clone,
    {
        type Output = T;

        fn roll(&mut self) -> Self::Output {
            self.0.clone()
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            HashMap::from([(self.0.clone(), 1.0)])
        }
    }

    #[derive(Debug)]
    pub struct Die<R: RngExt> {
        d: i32,
        rng: R,
    }

    impl<R: RngExt> Die<R> {
        pub fn new(d: i32, rng: R) -> Result<Self, ()> {
            if d > 0 { Ok(Self { d, rng }) } else { Err(()) }
        }
    }

    impl Die<ThreadRng> {
        pub fn new_baked(d: i32) -> Result<Self, ()> {
            if d > 0 {
                Ok(Self { d, rng: rng() })
            } else {
                Err(())
            }
        }
    }

    impl<R: RngExt> Roll for Die<R> {
        type Output = i32;

        fn roll(&mut self) -> Self::Output {
            self.rng.random_range(1..=self.d)
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

    #[macro_export]
    macro_rules! box_dice {
        ($t:ty; $($x:expr),* $(,)?) => {
            vec![$( Box::new($x) as Box<dyn Roll<Output = $t>> ),*]
        };
    }

    pub struct Dice<T> {
        dice: Vec<Box<dyn Roll<Output = T>>>,
    }

    impl<T> Dice<T> {
        pub fn new(dice: Vec<Box<dyn Roll<Output = T>>>) -> Self {
            Self { dice }
        }
    }

    impl<T> Roll for Dice<T>
    where
        T: Clone + Eq + Hash,
    {
        type Output = Vec<T>;

        fn roll(&mut self) -> Self::Output {
            self.dice.iter_mut().map(|die| die.roll()).collect()
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            let individual_stats: Vec<HashMap<T, f32>> =
                self.dice.iter().map(|die| die.get_stats()).collect();

            let mut out: HashMap<Vec<T>, f32> = HashMap::from([(vec![], 1.0)]);
            for individual_map in individual_stats {
                let existing = out;
                out = HashMap::new();
                for (key, cumulative_probability) in existing.iter() {
                    for (roll, probability) in individual_map.iter() {
                        let mut new_key = Vec::with_capacity(key.len() + 1);
                        new_key.extend_from_slice(key);
                        new_key.push(roll.clone());
                        out.insert(new_key, cumulative_probability * probability);
                    }
                }
            }
            out
        }
    }

    // #[derive(Debug)]
    pub struct AddDie<T, U, V>
    where
        U: Roll<Output: Add<V, Output = T>>,
    {
        d: MapDie<T, U, Box<dyn Fn(U::Output) -> T>>,
        _marker: PhantomData<V>,
    }

    impl<T, U, V> AddDie<T, U, V>
    where
        U: Roll<Output: Add<V, Output = T>>,
        V: Clone + 'static,
    {
        pub fn new(d: U, m: V) -> Self {
            Self {
                d: MapDie::new(Box::new(move |r: U::Output| r + m.clone()), d),
                _marker: PhantomData,
            }
        }
    }

    impl<T, U, V> Roll for AddDie<T, U, V>
    where
        T: Eq + Hash,
        U: Roll<Output: Add<V, Output = T> + Clone + Eq + Hash>,
    {
        type Output = T;

        fn roll(&mut self) -> Self::Output {
            self.d.roll()
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            self.d.get_stats()
        }
    }

    pub struct MulDie<T, U, V>
    where
        U: Roll<Output: Mul<V, Output = T>>,
    {
        d: MapDie<T, U, Box<dyn Fn(U::Output) -> T>>,
        _marker: PhantomData<V>,
    }

    impl<T, U, V> MulDie<T, U, V>
    where
        U: Roll<Output: Mul<V, Output = T>>,
        V: Clone + 'static,
    {
        pub fn new(d: U, m: V) -> Self {
            Self {
                d: MapDie::new(Box::new(move |r: U::Output| r * m.clone()), d),
                _marker: PhantomData,
            }
        }
    }

    impl<T, U, V> Roll for MulDie<T, U, V>
    where
        T: Eq + Hash,
        U: Roll<Output: Mul<V, Output = T> + Clone + Eq + Hash>,
    {
        type Output = T;

        fn roll(&mut self) -> Self::Output {
            self.d.roll()
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            self.d.get_stats()
        }
    }

    /// `MonoidSum` requires the following property:
    ///
    /// `sum(a, b, c, ...) == sum(sum(sum(sum(), a), b), c)...`
    ///
    /// "Monoid" usually requires an operation have an identity element and the associative property.
    ///
    /// `MonoidSum` does require an identity element but only requires the associative property
    /// in exactly the case stated above.
    pub trait MonoidSum: Sum {}

    /// `MonoidProduct` requires the following property:
    ///
    /// `product(a, b, c, ...) == product(product(product(product(), a), b), c)...`
    ///
    /// "Monoid" usually requires an operation have an identity element and the associative property.
    ///
    /// `MonoidProduct` does require an identity element but only requires the associative property
    /// in exactly the case stated above.
    pub trait MonoidProduct: Product {}

    macro_rules! impl_monoid_sum_product {
        // Match one or more types
        ($($t:ty)*) => {
            $(
                impl MonoidSum for $t {}
                impl MonoidProduct for $t {}
            )+
        };
    }

    impl_monoid_sum_product!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64);

    pub struct SumDie<T>
    where
        T: MonoidSum + Clone + Eq + Hash,
    {
        dice: Vec<Box<dyn Roll<Output = T>>>,
    }

    impl<T> SumDie<T>
    where
        T: MonoidSum + Clone + Eq + Hash,
    {
        pub fn new(dice: Vec<Box<dyn Roll<Output = T>>>) -> Self {
            Self { dice }
        }
    }

    impl<T> Roll for SumDie<T>
    where
        T: MonoidSum + Clone + Eq + Hash,
    {
        type Output = T;

        fn roll(&mut self) -> T {
            self.dice.iter_mut().map(|die| die.roll()).sum()
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            // Get the additive identity by summing an empty Vec<T>
            let additive_identity: T = Vec::<T>::new().into_iter().sum();
            let mut cumulative_stats = HashMap::from([(additive_identity, 1.0)]);

            // For each die we add up
            for d in self.dice.iter() {
                // Iterate on the map we've accumulated up to this point
                let old_stats;
                (old_stats, cumulative_stats) = (cumulative_stats, HashMap::new());

                // The old_stats map accounts for a sum of the elements of all the previous dice.
                // Now go one by one and fracture those probabilities by each value `d` could roll
                // and generate a new map.
                let d_stats = d.get_stats();

                for (d_result, d_prob) in d_stats.into_iter() {
                    for (cumulative_result, cumulative_prob) in old_stats.iter() {
                        let stat = cumulative_stats
                            .entry(
                                [d_result.clone(), cumulative_result.clone()]
                                    .into_iter()
                                    .sum(),
                            )
                            .or_insert(0.0);
                        *stat += d_prob * cumulative_prob;
                    }
                }
                // Notably this process is more efficient on later dice if the earlier dice
                // reuse more keys. Like d(2) + d(2) will have 3 keys (2, 3, 4), not 4.
            }
            cumulative_stats
        }
    }

    pub struct ProductDie<T>
    where
        T: MonoidProduct + Clone + Eq + Hash,
    {
        dice: Vec<Box<dyn Roll<Output = T>>>,
    }

    impl<T> ProductDie<T>
    where
        T: MonoidProduct + Clone + Eq + Hash,
    {
        pub fn new(dice: Vec<Box<dyn Roll<Output = T>>>) -> Self {
            Self { dice }
        }
    }

    impl<T> Roll for ProductDie<T>
    where
        T: MonoidProduct + Clone + Eq + Hash,
    {
        type Output = T;

        fn roll(&mut self) -> T {
            self.dice.iter_mut().map(|die| die.roll()).product()
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            // Get the multiplicative identity by summing an empty Vec<T>
            let multiplicative_identity: T = Vec::<T>::new().into_iter().product();
            let mut cumulative_stats = HashMap::from([(multiplicative_identity, 1.0)]);

            // For each die we multiply
            for d in self.dice.iter() {
                // Iterate on the map we've accumulated up to this point
                let old_stats;
                (old_stats, cumulative_stats) = (cumulative_stats, HashMap::new());

                // The old_stats map accounts for a product of the elements of all the previous dice.
                // Now go one by one and fracture those probabilities by each value `d` could roll
                // and generate a new map.
                let d_stats = d.get_stats();

                for (d_result, d_prob) in d_stats.into_iter() {
                    for (cumulative_result, cumulative_prob) in old_stats.iter() {
                        let stat = cumulative_stats
                            .entry(
                                [d_result.clone(), cumulative_result.clone()]
                                    .into_iter()
                                    .product(),
                            )
                            .or_insert(0.0);
                        *stat += d_prob * cumulative_prob;
                    }
                }
                // Notably this process is more efficient on later dice if the earlier dice
                // reuse more keys. Like d(2) + d(2) will have 3 keys (2, 3, 4), not 4.
            }
            cumulative_stats
        }
    }

    pub struct Advantage<T, T1, T2>
    where
        T: Ord,
        T1: Roll<Output = T>,
        T2: Roll<Output = T>,
    {
        dice: roll_tuple_type!(T1, T2),
    }

    impl<T, T1, T2> Advantage<T, T1, T2>
    where
        T: Ord,
        T1: Roll<Output = T>,
        T2: Roll<Output = T>,
    {
        pub fn new(d_1: T1, d_2: T2) -> Advantage<T, T1, T2> {
            Advantage {
                dice: roll_tuple!(d_1, d_2),
            }
        }
    }

    impl<T, T1, T2> CompoundRoll for Advantage<T, T1, T2>
    where
        T: Ord + Hash + Clone,
        T1: Roll<Output = T>,
        T2: Roll<Output = T>,
    {
        type Output = T;

        type DiceSet = roll_tuple_type!(T1, T2);

        fn get_dice(&self) -> &Self::DiceSet {
            &self.dice
        }

        fn get_dice_mut(&mut self) -> &mut Self::DiceSet {
            &mut self.dice
        }

        fn calculate(&self, results: <Self::DiceSet as RollTuple>::Outputs) -> Self::Output {
            let roll_tuple_pattern!(r1, r2) = results;
            Ord::max(r1, r2)
        }
    }

    pub struct Disadvantage<T, T1, T2>
    where
        T: Ord,
        T1: Roll<Output = T>,
        T2: Roll<Output = T>,
    {
        dice: roll_tuple_type!(T1, T2),
    }

    impl<T, T1, T2> Disadvantage<T, T1, T2>
    where
        T: Ord,
        T1: Roll<Output = T>,
        T2: Roll<Output = T>,
    {
        pub fn new(d_1: T1, d_2: T2) -> Disadvantage<T, T1, T2> {
            Disadvantage {
                dice: roll_tuple!(d_1, d_2),
            }
        }
    }

    impl<T, T1, T2> CompoundRoll for Disadvantage<T, T1, T2>
    where
        T: Ord + Hash + Clone,
        T1: Roll<Output = T>,
        T2: Roll<Output = T>,
    {
        type Output = T;

        type DiceSet = roll_tuple_type!(T1, T2);

        fn get_dice(&self) -> &Self::DiceSet {
            &self.dice
        }

        fn get_dice_mut(&mut self) -> &mut Self::DiceSet {
            &mut self.dice
        }

        fn calculate(&self, results: <Self::DiceSet as RollTuple>::Outputs) -> Self::Output {
            let roll_tuple_pattern!(r1, r2) = results;
            Ord::min(r1, r2)
        }
    }

    pub struct MapDie<T, R: Roll, M: Fn(R::Output) -> T> {
        dice: roll_tuple_type!(R),
        map: M,
        _marker: PhantomData<fn(R::Output) -> T>,
    }

    impl<T, R: Roll, M: Fn(R::Output) -> T> MapDie<T, R, M> {
        pub fn new(map: M, die: R) -> Self {
            Self {
                map,
                dice: roll_tuple!(die),
                _marker: PhantomData,
            }
        }
    }

    impl<T, R: Roll, M: Fn(R::Output) -> T> CompoundRoll for MapDie<T, R, M>
    where
        R::Output: Eq + Hash + Clone,
    {
        type Output = T;

        type DiceSet = roll_tuple_type!(R);

        fn calculate(&self, results: <Self::DiceSet as RollTuple>::Outputs) -> Self::Output {
            (self.map)(results.0)
        }

        fn get_dice(&self) -> &Self::DiceSet {
            &self.dice
        }

        fn get_dice_mut(&mut self) -> &mut Self::DiceSet {
            &mut self.dice
        }
    }

    // pub struct HighestDice {
    //     dice: Vec<Box<dyn Roll>>,
    //     count: usize,
    // }

    // impl HighestDice {
    //     pub fn new(dice: Vec<Box<dyn Roll>>, count: usize) -> Result<Self, usize> {
    //         if count > dice.len() {
    //             Err(dice.len())
    //         } else {
    //             Ok(Self {dice, count})
    //         }
    //     }
    // }

    // impl <T> CompoundRoll for HighestDice {
    //     fn c(&mut self) -> i32 {
    //         results.sort_by_key(|w| Reverse(*w));
    //         results.truncate(self.count);
    //         results
    //     }

    //     type Output=Vec<T>;

    //     type DiceSet;

    //     fn get_dice(&self) -> &Self::DiceSet {
    //         todo!()
    //     }

    //     fn get_dice_mut(&mut self) -> &mut Self::DiceSet {
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
