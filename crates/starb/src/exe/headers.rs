use std::ops::Range;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PeHeaders {
    // TODO: This is missing a lot, though the rest is never used
    pub entry_point: usize,
    pub sections: Vec<PeImageSection>,
}

#[derive(Clone, Debug)]
pub struct PeImageSection {
    // TODO: Same here
    pub name: String,
    pub section: Range<usize>,
}
