pub trait ThenTry {
    fn then_try<T, E>(self, f: impl FnOnce() -> Result<T, E>) -> Result<Option<T>, E>;
}

/// I get really tired of not being able to use `?` inside a closure in some combinator.
/// I also find calling `transpose` confusing, I always hover over the types to see why I 
/// did that. So, I fixed it. Tempted to release a crate called bubble-combinators and implement 
/// a bunch of combinators that bubble up errors for using ? on.
impl ThenTry for bool {
    fn then_try<T, E>(self, f: impl FnOnce() -> Result<T, E>) -> Result<Option<T>, E> {
        self.then(|| f()).transpose()
    }
}
