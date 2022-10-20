pub mod compact;

pub trait Patch {
    fn start(&self) {}

    fn update(&self) {}

    fn enable(&self) {}

    fn disable(&self) {}

    fn toggle(&self) {}
}
