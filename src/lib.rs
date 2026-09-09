pub mod roll {
    use std::collections::HashMap;
    pub trait Roll {
        type Output;

        fn roll(&mut self) -> Self::Output;

        fn get_stats(&self) -> HashMap<Self::Output, f32>;
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
