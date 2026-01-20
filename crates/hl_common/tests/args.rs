use hl_common::args::{AgeArg, LogLevel, SizeArg};
use hl_core::Level;

#[test]
fn log_level_conversion_matches_core() {
    assert_eq!(Level::from(LogLevel::Trace), Level::Trace);
    assert_eq!(Level::from(LogLevel::Info), Level::Info);
}

#[test]
fn size_arg_parses_units() {
    assert_eq!("1K".parse::<SizeArg>().unwrap().0, 1024);
    assert_eq!("2M".parse::<SizeArg>().unwrap().0, 2 * 1024 * 1024);
    assert!("invalid".parse::<SizeArg>().is_err());
}

#[test]
fn age_arg_parses_days() {
    assert_eq!("7d".parse::<AgeArg>().unwrap().0, 7);
    assert_eq!("14".parse::<AgeArg>().unwrap().0, 14);
    assert!("invalid".parse::<AgeArg>().is_err());
}
