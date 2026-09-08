pub mod roll {
    use std::collections::HashMap;
    pub trait Roll {
        fn roll(&mut self) -> i32;
    }
}

pub mod utils;