//! Log rendering touches colors, icons, templates, scoping, and inline styles — splitting
//! each concern into its own module prevents a single 2000-line formatting file.

mod color;
mod format;
pub mod highlight;
mod icon;
mod scope;
pub mod style;
mod tag;

pub use color::{Color, colorize, colorize_bg};
pub use format::{FormatSegment, FormatTemplate, FormatValues, Placeholder};
pub use highlight::inject_tags;
pub use icon::{IconSet, IconType};
pub use scope::ScopeConfig;
pub use style::{Segment, parse, render, render_plain, strip_tags};
pub use tag::{Alignment, TagConfig, Transform};
