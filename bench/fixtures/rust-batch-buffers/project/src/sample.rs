//! One reading from a monitored host, as received on the wire: `series timestamp value`.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub series: String,
    pub timestamp: u64,
    pub value: f64,
}

#[derive(Debug, PartialEq)]
pub enum SampleError {
    MissingField(&'static str),
    BadNumber(String),
}

impl fmt::Display for SampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SampleError::MissingField(name) => write!(f, "missing field `{name}`"),
            SampleError::BadNumber(text) => write!(f, "not a number: {text}"),
        }
    }
}

impl Sample {
    pub fn parse(line: &str) -> Result<Sample, SampleError> {
        let mut fields = line.split_whitespace();
        let series = fields.next().ok_or(SampleError::MissingField("series"))?;
        let ts = fields.next().ok_or(SampleError::MissingField("timestamp"))?;
        let value = fields.next().ok_or(SampleError::MissingField("value"))?;
        Ok(Sample {
            series: series.to_string(),
            timestamp: ts.parse().map_err(|_| SampleError::BadNumber(ts.to_string()))?,
            value: value.parse().map_err(|_| SampleError::BadNumber(value.to_string()))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_line() {
        let s = Sample::parse("web-3.cpu 1700000000 0.75").unwrap();
        assert_eq!(s.series, "web-3.cpu");
        assert_eq!(s.timestamp, 1_700_000_000);
        assert_eq!(s.value, 0.75);
    }

    #[test]
    fn reports_missing_and_malformed_fields() {
        assert_eq!(Sample::parse("web-3.cpu"), Err(SampleError::MissingField("timestamp")));
        assert_eq!(Sample::parse("a 12 x"), Err(SampleError::BadNumber("x".into())));
    }
}
