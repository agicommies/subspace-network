pub mod linear;
pub mod yuma;

#[derive(Debug)]
#[allow(dead_code)]
pub enum EmissionError {
    EmittedMoreThanExpected { emitted: u64, expected: u64 },
    HasEmissionRemaining { emitted: u64 },
    Other(&'static str),
}

impl From<&'static str> for EmissionError {
    fn from(v: &'static str) -> Self {
        Self::Other(v)
    }
}
