//! Formatting and styling utilities for log output.

mod color;
mod format;
mod icon;
pub mod style;
mod tag;

pub use color::{Color, colorize, colorize_bg};
pub use format::{FormatSegment, FormatTemplate, FormatValues, Placeholder};
pub use icon::{IconSet, IconType};
pub use style::{Segment, parse, render, render_plain, strip_tags};
pub use tag::{Alignment, TagConfig, Transform};
