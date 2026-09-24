pub mod roll {
    use std::collections::HashMap;

    /// Provides a random element and its probability distribution.
    ///
    /// The `Roll::roll` method returns a random element and `Roll::get_stats`
    /// returns the probability of each output in the range of 0.0-1.0.
    ///
    /// # Example
    /// ```
    /// use dice::prelude::*;
    /// use dice::utils::dice::Advantage;
    ///
    /// let mut d20_1 = d(20);
    /// let mut d20_2 = d(20);
    /// let mut adv = Advantage::new(d20_1, d20_2);
    ///
    /// // How to use Roll::roll
    /// println!("Roll with advantage: {}", adv.roll());
    /// println!();
    ///
    /// // How to use Roll::get_stats
    /// let results = adv.get_stats();
    /// let mut results = results.into_iter().collect::<Vec<(i32, f32)>>();
    /// results.sort_by_key(|k| k.0);
    ///
    /// println!("Probability of each possible roll:");
    /// for (a, b) in results.iter() {
    ///     println!("{},{}", a, b.clone() * 100.0);
    /// }
    /// ```
    pub trait Roll {
        type Output;

        fn roll(&mut self) -> Self::Output;

        fn get_stats(&self) -> HashMap<Self::Output, f32>;
    }

    pub mod compound {
        use super::Roll;
        use std::{collections::HashMap, hash::Hash};

        pub trait RollTuple {
            type Outputs: Eq + Hash + Clone;

            fn roll_all(&mut self) -> Self::Outputs;

            fn get_stats_all(&self) -> HashMap<Self::Outputs, f32>;
        }

        /// Generate a folded tuple of [Roll] objects that implement [RollTuple]
        /// 
        /// Used alongside [roll_tuple_type] and [roll_tuple_pattern] to add convenience to
        /// implementing [CompoundRoll].
        /// 
        /// See [CompoundRoll] for examples.
        #[macro_export]
        macro_rules! roll_tuple {
            // Base case: nothing left → unit
            () => { () };

            // Recursive case: first element + rest
            ($head:expr $(, $tail:expr)*) => {
                ($head, roll_tuple!($($tail),*))
            };
        }

        #[macro_export]
        macro_rules! roll_tuple_type {
            // Base case: nothing left → unit
            () => { () };

            // Recursive case: first element + rest
            ($head:ty $(, $tail:ty)*) => {
                ($head, roll_tuple_type!($($tail),*))
            };
        }

        #[macro_export]
        macro_rules! roll_tuple_pattern {
            // Base case: nothing left → unit
            () => { () };

            // Recursive case: first element + rest
            ($head:pat $(, $tail:pat)*) => {
                ($head, roll_tuple_pattern!($($tail),*))
            };
        }

        impl RollTuple for () {
            type Outputs = ();

            fn roll_all(&mut self) -> Self::Outputs {
                ()
            }

            fn get_stats_all(&self) -> HashMap<Self::Outputs, f32> {
                HashMap::from([((), 1.0)])
            }
        }

        impl<H: Roll, T: RollTuple> RollTuple for (H, T)
        where
            H::Output: Eq + Hash + Clone,
        {
            type Outputs = (H::Output, T::Outputs);

            fn roll_all(&mut self) -> Self::Outputs {
                (self.0.roll(), self.1.roll_all())
            }

            fn get_stats_all(&self) -> HashMap<Self::Outputs, f32> {
                let predecessor = self.1.get_stats_all();
                let mut stats = HashMap::new();
                for (key, value) in self.0.get_stats().into_iter() {
                    for (key2, value2) in predecessor.iter() {
                        stats.insert((key.clone(), key2.clone()), value * value2);
                    }
                }
                stats
            }
        }

        /// A convenience trait to implement [Roll] for types that compound other [Roll] objects.
        /// 
        /// # Examples
        /// 
        /// A `PrintDie` struct that logs every roll to the terminal.
        /// 
        /// ```
        /// use dice::{
        ///     roll::{Roll, compound::*},
        ///     roll_tuple_type,
        ///     roll_tuple_pattern,
        /// };
        /// use std::fmt::Display;
        /// 
        /// struct PrintDie<T: Roll>
        /// where
        ///     T::Output: Display,
        /// {
        ///     d: roll_tuple_type!(T),
        /// }
        /// 
        /// impl<T: Roll<Output = i32>> CompoundRoll for PrintDie<T> {
        ///     type Output = i32;
        /// 
        ///     type DiceSet = roll_tuple_type!(T);
        /// 
        ///     fn get_dice(&self) -> &Self::DiceSet {
        ///         &self.d
        ///     }
        /// 
        ///     fn get_dice_mut(&mut self) -> &mut Self::DiceSet {
        ///         &mut self.d
        ///     }
        /// 
        ///     fn calculate(&self, results: <Self::DiceSet as RollTuple>::Outputs) -> Self::Output {
        ///         let roll_tuple_pattern!(roll) = results;
        ///         println!("Rolled {roll}");
        ///         roll
        ///     }
        /// }
        /// ```
        pub trait CompoundRoll {
            type Output;
            type DiceSet: RollTuple;

            fn get_dice(&self) -> &Self::DiceSet;

            fn get_dice_mut(&mut self) -> &mut Self::DiceSet;

            // Must be deterministic
            fn calculate(&self, results: <Self::DiceSet as RollTuple>::Outputs) -> Self::Output;
        }

        impl<T: CompoundRoll> Roll for T
        where
            T::Output: Eq + Hash,
        {
            type Output = T::Output;

            fn roll(&mut self) -> Self::Output {
                let results = self.get_dice_mut().roll_all();
                self.calculate(results)
            }

            fn get_stats(&self) -> HashMap<Self::Output, f32> {
                let uncalculated_stats = self.get_dice().get_stats_all();
                let mut stats = HashMap::new();
                for (key, value) in uncalculated_stats.into_iter() {
                    let stat = stats.entry(self.calculate(key)).or_insert(0.0);
                    *stat += value;
                }
                stats
            }
        }
    }
}

pub mod utils;

pub mod prelude {
    pub use crate::box_dice;
    pub use crate::roll::Roll;
    pub use crate::utils::dice::Die;
    pub use crate::{roll_tuple, roll_tuple_pattern, roll_tuple_type};

    use rand::rngs::ThreadRng;

    /// Convenience method to generate a simple die.
    ///
    /// Unwraps the result of `Die::new_baked(n)`.
    ///
    /// # Panics
    ///
    /// Panics if `n <= 0`, since a die chooses a random integer in a range with a least element of one.
    ///
    /// # Examples
    ///
    /// ```
    /// use dice::{
    ///     roll::Roll,
    ///     prelude::d,
    ///     utils::dice::AddDie,
    /// };
    ///
    /// let player_strength_modifier = 4;
    /// 
    /// // Use the `d` function to quickly make a modified die
    /// let mut strength_die = AddDie::new(d(20), player_strength_modifier);
    ///
    /// for _ in 0..20 {
    ///     println!("Rolled {} for strength.", strength_die.roll());
    /// }
    /// ```
    ///
    /// ```
    /// use dice::{
    ///     roll::Roll,
    ///     prelude::d,
    ///     utils::dice::Advantage,
    /// };
    ///
    /// // Use the `d` function to quickly make an advantage die
    /// let mut adv = Advantage::new(d(20), d(20));
    /// let advantage_statistics = adv.get_stats();
    /// ```
    pub fn d(n: i32) -> Die<ThreadRng> {
        Die::new_baked(n).unwrap()
    }
}
