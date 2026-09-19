pub mod roll {
    use std::{collections::HashMap, hash::Hash};
    pub trait Roll {
        type Output;

        fn roll(&mut self) -> Self::Output;

        fn get_stats(&self) -> HashMap<Self::Output, f32>;
    }

    pub trait RollTuple {
        type Outputs: Eq + Hash + Clone;

        fn roll_all(&mut self) -> Self::Outputs;

        fn get_stats_all(&self) -> HashMap<Self::Outputs, f32>;
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

    // This is the trait someone will manually implement
    pub trait CompoundRoll {
        type Output;
        type DiceSet: RollTuple;

        fn get_stats_all(&self) -> HashMap<<Self::DiceSet as RollTuple>::Outputs, f32>;

        fn roll_all(&mut self) -> <Self::DiceSet as RollTuple>::Outputs;

        // Must be deterministic
        fn calculate(&self, results: <Self::DiceSet as RollTuple>::Outputs) -> Self::Output;
    }

    impl<T: CompoundRoll> Roll for T
    where
        T::Output: Eq + Hash,
    {
        type Output = T::Output;

        fn roll(&mut self) -> Self::Output {
            let results = self.roll_all();
            self.calculate(results)
        }

        fn get_stats(&self) -> HashMap<Self::Output, f32> {
            let uncalculated_stats = self.get_stats_all();
            let mut stats = HashMap::new();
            for (key, value) in uncalculated_stats.into_iter() {
                let stat = stats.entry(self.calculate(key)).or_insert(0.0);
                *stat += value;
            }
            stats
        }
    }
}

pub mod utils;

pub mod prelude {
    pub use rand::rngs::ThreadRng;

    pub use crate::roll::Roll;
    pub use crate::utils::dice::Die;

    pub fn d(n: i32) -> Die<ThreadRng> {
        Die::new_baked(n)
    }
}
