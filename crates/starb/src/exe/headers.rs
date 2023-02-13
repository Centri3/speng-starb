use std::ops::Range;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PeHeaders {
    // TODO(Centri3): This is missing a lot, though the rest is never used
    entry_point: usize,
    sections: Vec<PeImageSection>,
}

#[derive(Clone, Debug)]
struct PeImageSection {
    // TODO(Centri3): Same here
    name: &'static str,
    section: Range<usize>,
}
