pub trait IndexedPosition<I, P> {
    fn get_position(&self, index: usize) -> P;
    fn get_position_fixed(&self, index: I) -> P;
    fn get_fixed_index(&self, position: P) -> I;
    fn get_index(&self, position: P) -> usize;
}
