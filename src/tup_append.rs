pub trait TupAppend<T, Result> {
    fn tup_append(self, x: T) -> Result;
}
 
impl<A, B> TupAppend<B, (A,B)> for (A,) {
    fn tup_append(self, x: B) -> (A, B) {
        (self.0, x)
    }
}
 
impl<A, B, C> TupAppend<C, (A,B,C)> for (A, B) {
    fn tup_append(self, x: C) -> (A, B, C) {
        (self.0, self.1, x)
    }
}

impl<A, B, C, D> TupAppend<D, (A,B,C,D)> for (A, B, C) {
    fn tup_append(self, x: D) -> (A, B, C, D) {
        (self.0, self.1, self.2, x)
    }
}

impl<A, B, C, D, E> TupAppend<E, (A,B,C,D,E)> for (A, B, C, D) {
    fn tup_append(self, x: E) -> (A, B, C, D, E) {
        (self.0, self.1, self.2, self.3, x)
    }
}

impl<A, B, C, D, E, F> TupAppend<F, (A,B,C,D,E,F)> for (A, B, C, D, E) {
    fn tup_append(self, x: F) -> (A, B, C, D, E, F) {
        (self.0, self.1, self.2, self.3, self.4, x)
    }
}

// TODO possibly need longer TupAppend