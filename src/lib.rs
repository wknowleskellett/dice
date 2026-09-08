pub mod utils;

pub mod roll {
    use rand::Rng;

    pub trait Roll {
        fn roll(&mut self) -> i32;
    
        fn repeat(&mut self, n: i32) -> i32 {
            (1..n).map(|_| self.roll()).sum()
        }
    }

    pub trait RollRng {
        fn roll_rng<U: Rng>(&self, rng: &mut U) -> i32;
    }
    
    pub trait CriticalSuccess {
        fn is_critical_success(&self, n: i32) -> bool;
    }
    
    pub trait RollCriticalSuccess: Roll + CriticalSuccess {
        fn roll_critical_success(&mut self) -> (i32, bool) {
            let result = self.roll();
            (result, self.is_critical_success(result))
        }
    }

    impl <T: Roll + CriticalSuccess> RollCriticalSuccess for T {}

    pub trait CriticalFailure {
        fn is_critical_failure(&self, n: i32) -> bool;
    }
    
    pub trait RollCriticalFailure: Roll + CriticalFailure {
        fn roll_critical_failure(&mut self) -> (i32, bool) {
            let result = self.roll();
            (result, self.is_critical_failure(result))
        }
    }

    impl <T: Roll + CriticalFailure> RollCriticalFailure for T {}
    
    pub trait RollCrit: RollCriticalFailure + RollCriticalSuccess {
        fn roll_crit(&mut self) -> (i32, bool, bool) {
            let result = self.roll();
            (result, self.is_critical_failure(result), self.is_critical_success(result))
        }
    }

    impl <T: Roll + CriticalFailure + CriticalSuccess> RollCrit for T {}

    pub trait Explosion: Roll {
        fn explodes(&self, n: i32) -> bool;
    }

    pub trait RollExplosion: Roll + Explosion {

        fn roll_explosion(&mut self) -> i32 {
            let mut exploded = true;
            let mut result;
            let mut sum = 0;
            while exploded {
                result = self.roll();
                sum += result;
                exploded = self.explodes(result);
            }
            sum
        }
    }
}