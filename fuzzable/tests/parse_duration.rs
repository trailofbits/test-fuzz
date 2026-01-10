// smoelius: This example is due to @disconnect3d.
// See: https://rustsec.org/advisories/RUSTSEC-2021-0041.html

use parse_duration::parse::Error;
use std::time::Duration;

#[test_fuzz::test_fuzz]
fn parse(input: &str) -> Result<Duration, Error> {
    parse_duration::parse(input)
}

#[test]
fn test() {
    parse("1e5 ns").unwrap();
}
