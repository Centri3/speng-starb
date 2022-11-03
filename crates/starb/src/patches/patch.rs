pub trait Patch {
    fn start(&self);

    fn enable(&self);

    fn disable(&self);

    fn toggle(&self) {}

    fn update(&self) {}
}
