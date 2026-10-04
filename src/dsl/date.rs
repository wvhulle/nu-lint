use std::sync::LazyLock;

use regex::Regex;

const SHARED_CONVERSIONS: &str = "YmdHMSybBaAjeIpzsFTDRuwUWVGghklPnt%";

static CONVERSION_SPECIFIER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%[-_0:]*(.)?").unwrap());

pub fn chrono_compatible_format(operand: &str) -> Option<&str> {
    let format = operand.strip_prefix('+')?;
    CONVERSION_SPECIFIER
        .captures_iter(format)
        .all(|specifier| {
            specifier
                .get(1)
                .is_some_and(|conversion| SHARED_CONVERSIONS.contains(conversion.as_str()))
        })
        .then_some(format)
}
