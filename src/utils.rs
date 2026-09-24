#[cfg(doc)]
use crate::roll::Roll;

/// This module provides common implementations of the [Roll] trait.
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

    /// A convenience [Roll] object with a single possible result
    ///
    /// A stand-in type for a case where a [Roll] type is required but a constant result is sufficient.
    ///
    /// # Example
    /// ```
    /// use dice::{prelude::d, roll::Roll, utils::dice::ConstRoll};
    ///
    /// enum NumRoller {
    ///     Num(i32),
    ///     Sides(i32),
    /// }
    ///
    /// impl NumRoller {
    ///     fn get(&self) -> Box<dyn Roll<Output=i32>> {
    ///         match self {
    ///             NumRoller::Num(n) => Box::new(ConstRoll::new(*n)),
    ///             NumRoller::Sides(n) => Box::new(d(*n)),
    ///         }
    ///     }
    /// }
    /// ```
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

    /// A fair coin
    ///
    /// # Examples
    /// ```
    /// use dice::{roll::Roll, utils::dice::Coin};
    /// use rand::rng;
    ///
    /// let mut go_to_store = Coin::new(rng());
    /// let mut singing_to_myself = Coin::new_baked();
    ///
    /// println!("I {} going to the store", if go_to_store.roll() { "am" } else {"am not"} );
    /// println!("I {} singing to myself", if singing_to_myself.roll() { "am" } else {"am not"} );
    /// ```
    pub struct Coin<R: RngExt> {
        rng: R,
    }

    impl<R: RngExt> Coin<R> {
        pub fn new(rng: R) -> Self {
            Self { rng }
        }
    }

    impl Coin<ThreadRng> {
        pub fn new_baked() -> Self {
            Self { rng: rng() }
        }
    }

    impl<R: RngExt> Roll for Coin<R> {
        type Output = bool;

        fn roll(&mut self) -> Self::Output {
            self.rng.random_bool(0.5)
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            HashMap::from([(true, 0.5), (false, 0.5)])
        }
    }

    /// An unweighted die with integer sides
    ///
    /// If you don't need to use your own [Rng](rand::Rng) implementation,
    /// consider using [dice::prelude::d](crate::prelude::d).
    ///
    /// # Examples
    ///
    /// ```
    /// use rand::rng;
    /// use dice::{roll::Roll, utils::dice::Die};
    ///
    /// let mut d10 = Die::new(10, rng()).unwrap(); // Only errors when the number is 0 or less
    /// println!("I got a {}", d10.roll());
    /// ```
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

    /// A rollable [Vec] of [Roll] objects.
    ///
    /// It is recommended to utilize the [box_dice] macro for this use case.
    ///
    /// # Examples
    /// ```
    /// use dice::{
    ///     prelude::d,
    ///     roll::{Roll, box_dice},
    ///     utils::dice::{Dice, MulDie},
    /// };
    /// let dice_list =
    ///     box_dice![i32; d(4), d(6), d(8), d(10), MulDie::new(d(10), 10), d(12), d(20)];
    /// let mut my_dice = Dice::new(dice_list);
    ///
    /// for (i, die_result) in my_dice.roll().iter().enumerate() {
    ///     println!("Die {} rolled {}", i + 1, die_result);
    /// }
    /// ```
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

    /// Add a constant to a die result
    ///
    /// The result of the die roll is added to the constant. This works for any types
    /// `U`, `V` where `U::Output` can be added to `V`. `T` is the output type.
    ///
    /// # Examples
    /// ```
    /// use dice::{prelude::d, roll::Roll, utils::dice::AddDie};
    ///
    /// let mut d10 = d(10);
    /// let mut d10_plus_five = AddDie::new(d(10), 5);
    ///
    /// println!("d10 results:");
    /// for _ in 0..10 {
    ///     println!("{}", d10.roll());
    /// }
    ///
    /// println!("d10+5 results:");
    /// for _ in 0..10 {
    ///     println!("{}", d10_plus_five.roll());
    /// }
    /// ```
    pub struct AddDie<T, U, V>
    where
        U: Roll<Output: Add<V, Output = T>>,
    {
        d: MapDie<T, U, Box<dyn Fn(U::Output) -> T>>,
        _marker: PhantomData<V>,
    }

    impl<T, U, V> AddDie<T, U, V>
    where
        U: Roll<Output: Add<V, Output = T> + Clone + Eq + Hash>,
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

    /// Multiply a die result by a constant
    ///
    /// The result of the die roll is multiplied by the constant. This works for any types
    /// `U`, `V` where `U::Output` can be multiplied by `V`. `T` is the output type.
    ///
    /// # Examples
    /// ```
    /// use dice::{prelude::d, roll::Roll, utils::dice::MulDie};
    ///
    /// let mut d10 = d(10);
    /// let mut d10_times_five = MulDie::new(d(10), 5);
    ///
    /// println!("d10 results:");
    /// for _ in 0..10 {
    ///     println!("{}", d10.roll());
    /// }
    ///
    /// println!("d10*5 results:");
    /// for _ in 0..10 {
    ///     println!("{}", d10_times_five.roll());
    /// }
    /// ```
    pub struct MulDie<T, U, V>
    where
        U: Roll<Output: Mul<V, Output = T>>,
    {
        d: MapDie<T, U, Box<dyn Fn(U::Output) -> T>>,
        _marker: PhantomData<V>,
    }

    impl<T, U, V> MulDie<T, U, V>
    where
        U: Roll<Output: Mul<V, Output = T> + Clone + Eq + Hash>,
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

    /// Used to create a SumDie
    ///
    /// This trait is implemented by default on all numeric types.
    ///
    /// `MonoidSum` requires the following property:
    ///
    /// `sum(a, b, c, ...) == sum(sum(sum(sum(), a), b), c)...`
    ///
    /// "Monoid" usually requires an operation have an identity element and the associative property.
    ///
    /// `MonoidSum` does require an identity element but only requires the associative property
    /// in exactly the case stated above.
    pub trait MonoidSum: Sum {}

    /// Used to create a ProductDie
    ///
    /// This trait is implemented by default on all numeric types.
    ///
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

    /// Sum the results of the input dice
    ///
    /// Consider initializing this with the [box_dice] macro.
    ///
    /// To use this on a custom type, implement [Sum] and [MonoidSum]. The [Sum] implementation
    /// must meet the additional requirements of [MonoidSum].
    ///
    /// # Examples
    ///
    /// ```
    /// use dice::{
    ///     prelude::d,
    ///     roll::{Roll, box_dice},
    ///     utils::dice::{ConstRoll, SumDie},
    /// };
    ///
    /// let mut sum = SumDie::new(box_dice!(
    ///     i32;
    ///     d(6),
    ///     d(6),
    ///     d(6),
    ///     ConstRoll::new(10),
    /// ));
    /// for _ in 0..10 {
    ///     println!("I got {}", sum.roll());
    /// }
    /// ```
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

    /// Get the product of the results of the input dice
    ///
    /// Consider initializing this with the [box_dice] macro.
    ///
    /// To use this on a custom type, implement [Product] and [MonoidProduct]. The [Product] implementation
    /// must meet the additional requirements of [MonoidProduct].
    ///
    /// # Examples
    ///
    /// ```
    /// use dice::{
    ///     prelude::d,
    ///     roll::{Roll, box_dice},
    ///     utils::dice::{ConstRoll, ProductDie},
    /// };
    ///
    /// let mut product = ProductDie::new(box_dice!(
    ///     i32;
    ///     d(6),
    ///     d(6),
    ///     d(6),
    ///     ConstRoll::new(10),
    /// ));
    /// for _ in 0..10 {
    ///     println!("I got {}", product.roll());
    /// }
    /// ```
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

    /// The greater of two orderable dice
    ///
    /// # Examples
    /// ```
    /// use dice::{prelude::d, roll::Roll, utils::dice::Advantage};
    ///
    /// let mut adv = Advantage::new(d(20), d(20));
    ///
    /// for _ in 0..10 {
    ///     println!("{}", adv.roll());
    /// }
    /// ```
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

    /// The lesser of two orderable dice
    ///
    /// # Examples
    /// ```
    /// use dice::{prelude::d, roll::Roll, utils::dice::Advantage};
    ///
    /// let mut adv = Advantage::new(d(20), d(20));
    ///
    /// for _ in 0..10 {
    ///     println!("{}", adv.roll());
    /// }
    /// ```
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

    /// A map imposed on a die result
    ///
    /// This is a convenience implementation. If you have a function
    /// to run on the results of one die to produce a new die, this is the place to use it.
    ///
    /// This die is used by [AddDie] and [MulDie]. It is syntax sugar for implementing a [CompoundRoll]
    /// type composed of a single die.
    ///
    /// # Example
    ///
    /// ```
    /// use dice::{prelude::d, roll::Roll, utils::dice::MapDie};
    ///
    /// let sqrt_int = |n| ((n as f32).sqrt()*100.0) as i32;
    /// let mut sqrt_d100 = MapDie::new(sqrt_int, d(100));
    ///
    /// for _ in 0..100 {
    ///     println!("{:.2}", (sqrt_d100.roll() as f32)/100.0);
    /// }
    /// ```
    pub struct MapDie<T, R: Roll, M: Fn(R::Output) -> T> {
        dice: roll_tuple_type!(R),
        map: M,
        _marker: PhantomData<fn(R::Output) -> T>,
    }

    impl<T, R: Roll, M: Fn(R::Output) -> T> MapDie<T, R, M>
    where
        R::Output: Eq + Hash + Clone,
    {
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
}
