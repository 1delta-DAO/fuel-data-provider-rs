pub trait Entity<M> {
    fn from_model(model: &M) -> Self;
    fn to_model(&self) -> M;
}
