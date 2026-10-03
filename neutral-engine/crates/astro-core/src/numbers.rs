//! Normalize JSON integers by decimal value, without float rounding or truncation.
use serde::{de::Error, Deserialize, Deserializer};
use serde_json::value::RawValue;

fn exact_integer(raw: &str) -> Result<i64, &'static str> {
    let raw = raw.trim();
    let (negative, unsigned) = match raw.strip_prefix('-') {
        Some(value) => (true, value),
        None => (false, raw),
    };
    if !unsigned.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        return Err("Expected a JSON number with an integer value");
    }
    // RawValue has already validated JSON number syntax.
    let (mantissa, exponent) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, "0"), |(m, e)| (m, e));
    let decimals = mantissa.split_once('.').map_or(0, |(_, f)| f.len());
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let significant = digits.trim_matches('0');
    if significant.is_empty() {
        return Ok(0);
    }
    let trailing_zeros = digits.len() - digits.trim_end_matches('0').len();
    let scale = exponent
        .parse::<i64>()
        .ok()
        .and_then(|e| e.checked_sub(decimals as i64))
        .and_then(|e| e.checked_add(trailing_zeros as i64))
        .ok_or("Integer exponent is outside the supported range")?;
    if scale < 0 {
        return Err("Expected an integer value; fractional numbers are not accepted");
    }
    // Both callers use 32-bit types, requiring at most ten significant digits.
    if scale > 10 || significant.len() as i64 + scale > 10 {
        return Err("Integer is outside the supported 32-bit range");
    }
    let canonical = format!(
        "{}{}{}",
        if negative { "-" } else { "" },
        significant,
        "0".repeat(scale as usize)
    );
    canonical.parse().map_err(|_| "Invalid integer value")
}

pub(super) fn i32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i32, D::Error> {
    let raw = Box::<RawValue>::deserialize(deserializer)?;
    let value = exact_integer(raw.get()).map_err(D::Error::custom)?;
    value
        .try_into()
        .map_err(|_| D::Error::custom("Integer is outside the i32 range"))
}

pub(super) fn optional_u32<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u32>, D::Error> {
    let raw = Option::<Box<RawValue>>::deserialize(deserializer)?;
    raw.map(|raw| {
        let value = exact_integer(raw.get()).map_err(D::Error::custom)?;
        value
            .try_into()
            .map_err(|_| D::Error::custom("Integer is outside the u32 range"))
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Integer(#[serde(deserialize_with = "i32")] i32);
    #[test]
    fn integer_notation_preserves_exact_value() {
        for (raw, expected) in [
            ("2000", 2000),
            ("2000.0", 2000),
            ("2e3", 2000),
            ("20000e-1", 2000),
            ("2000.000000000000000000", 2000),
            ("-0.0", 0),
            ("-2147483648.0", i32::MIN),
            ("2147483647e0", i32::MAX),
            ("0e999999999999999999999999999999", 0),
        ] {
            assert_eq!(
                serde_json::from_str::<Integer>(raw).unwrap().0,
                expected,
                "{raw}"
            );
        }
        for raw in [
            "2000.5",
            "2000.0000000000000001",
            "1999.9999999999999999",
            "1e-100",
            "2147483648.0",
            "-2147483649",
            "1e999999999999999999999",
            "\"2000\"",
            "true",
            "null",
            "[]",
            "{}",
        ] {
            assert!(serde_json::from_str::<Integer>(raw).is_err(), "{raw}");
        }
    }
}
