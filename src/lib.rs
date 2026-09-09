pub mod roll {
    use std::collections::HashMap;
    pub trait Roll {
        fn roll(&mut self) -> i32;

        fn get_stats(&self) -> HashMap<i32, i32>;
    }
}

pub mod utils;

pub mod prelude {
    use rand::rngs::ThreadRng;

    use crate::utils::dice::Die;

    pub fn d(n: i32) -> Die<ThreadRng> {
        Die::new_baked(n)
    }
}
