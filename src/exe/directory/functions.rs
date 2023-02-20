use crate::exe::headers::HEADERS;
use eyre::Result;
use std::ops::Range;

pub type Functions = Vec<Function>;

#[derive(Debug)]
pub struct Function(pub Range<usize>);

impl Function {
    #[inline]
    #[instrument()]
    pub fn name(&self) -> Result<String> {
        trace!("Getting name of function");

        Ok(format!(
            "fn_{:x}",
            HEADERS.optional()?.image_base() + self.0.start
        ))
    }
}
