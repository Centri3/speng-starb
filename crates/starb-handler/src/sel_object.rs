pub trait Object {
    fn yeah(&self) {
        println!("the first")
    }
}

pub struct Galaxy {}

impl Object for Galaxy {
    fn yeah(&self) {
        println!("custom galaxy message")
    }
}

pub struct Planet {}

impl Object for Planet {}

pub struct SelObject<T>(pub T)
where
    T: Object;

impl SelObject<Galaxy> {
    pub fn yeah(&self) {
        println!("yeah");
    }
}

impl SelObject<Planet> {
    pub fn yeah_two(&self) {
        println!("yeah_two");
    }
}
