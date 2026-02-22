//! Different outputs need different column layouts — terminal may want `{tag} {scope} {msg}`
//! while file output needs `{timestamp} {tag} {scope} {msg}`. Templates make this user-configurable
//! instead of hardcoded per backend.

/// Closed set of known substitution tokens — unknown `{names}` pass through as literal text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placeholder {
    Tag,
    Icon,
    Scope,
    Msg,
    Timestamp,
    Level,
    App,
    Year,
    Month,
    Day,
}

impl Placeholder {
    /// Template parsing needs to match brace-delimited names against known placeholders.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tag => "tag",
            Self::Icon => "icon",
            Self::Scope => "scope",
            Self::Msg => "msg",
            Self::Timestamp => "timestamp",
            Self::Level => "level",
            Self::App => "app",
            Self::Year => "year",
            Self::Month => "month",
            Self::Day => "day",
        }
    }

    /// Iteration over all variants avoids forgetting a placeholder when matching by name.
    pub const ALL: &'static [Self] = &[
        Self::Tag,
        Self::Icon,
        Self::Scope,
        Self::Msg,
        Self::Timestamp,
        Self::Level,
        Self::App,
        Self::Year,
        Self::Month,
        Self::Day,
    ];
}

/// Parsing into segments once avoids re-scanning the template on every log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatSegment {
    /// Whitespace, separators, and unknown `{names}` pass through untouched.
    Literal(String),
    /// Known tokens are substituted with formatted values at render time.
    Placeholder(Placeholder),
}

/// Pre-parsed template avoids string scanning on every log call — parse once, render many.
#[derive(Debug, Clone)]
pub struct FormatTemplate {
    segments: Vec<FormatSegment>,
}

impl FormatTemplate {
    /// One-time parse turns `"{tag} {scope} {msg}"` into a segment list for fast repeated rendering.
    #[must_use]
    pub fn parse(template: &str) -> Self {
        let mut segments = Vec::new();
        let mut current = String::new();
        let mut i = 0;
        let chars: Vec<char> = template.chars().collect();

        while i < chars.len() {
            if chars[i] == '{' {
                // Look for closing brace
                if let Some(end) = chars[i..].iter().position(|&c| c == '}') {
                    let end = i + end;
                    let name: String = chars[i + 1..end].iter().collect();

                    // Save any accumulated literal
                    if !current.is_empty() {
                        segments.push(FormatSegment::Literal(current.clone()));
                        current.clear();
                    }

                    // Check if it's a known placeholder
                    if let Some(ph) = Self::match_placeholder(&name) {
                        segments.push(FormatSegment::Placeholder(ph));
                    } else {
                        // Unknown placeholder, keep as literal
                        segments.push(FormatSegment::Literal(format!("{{{name}}}")));
                    }

                    i = end + 1;
                    continue;
                }
            }

            current.push(chars[i]);
            i += 1;
        }

        if !current.is_empty() {
            segments.push(FormatSegment::Literal(current));
        }

        Self { segments }
    }

    fn match_placeholder(name: &str) -> Option<Placeholder> {
        for ph in Placeholder::ALL {
            if ph.as_str() == name {
                return Some(*ph);
            }
        }
        None
    }

    /// Tests and downstream code need direct access to verify parse results.
    #[must_use]
    pub fn segments(&self) -> &[FormatSegment] {
        &self.segments
    }

    /// Substitutes formatted values into the pre-parsed segments — the hot path for every log line.
    #[must_use]
    pub fn render(&self, values: &FormatValues) -> String {
        let mut result = String::new();

        for segment in &self.segments {
            match segment {
                FormatSegment::Literal(s) => result.push_str(s),
                FormatSegment::Placeholder(ph) => {
                    let value = match ph {
                        Placeholder::Tag => &values.tag,
                        Placeholder::Icon => &values.icon,
                        Placeholder::Scope => &values.scope,
                        Placeholder::Msg => &values.msg,
                        Placeholder::Timestamp => &values.timestamp,
                        Placeholder::Level => &values.level,
                        Placeholder::App => &values.app,
                        Placeholder::Year => &values.year,
                        Placeholder::Month => &values.month,
                        Placeholder::Day => &values.day,
                    };
                    result.push_str(value);
                }
            }
        }

        result
    }
}

impl Default for FormatTemplate {
    fn default() -> Self {
        Self::parse("{tag} {scope}  {msg}")
    }
}

/// Typed value bag ensures every placeholder has a corresponding field — no risk of key typos at runtime.
#[derive(Debug, Clone, Default)]
pub struct FormatValues {
    pub tag: String,
    pub icon: String,
    pub scope: String,
    pub msg: String,
    pub timestamp: String,
    pub level: String,
    pub app: String,
    pub year: String,
    pub month: String,
    pub day: String,
}

impl FormatValues {
    /// Empty defaults let callers set only the fields they need without boilerplate for the rest.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The `{tag}` placeholder needs a pre-formatted level indicator (e.g., `[INFO]`).
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// The `{icon}` placeholder needs the glyph string for the current level and icon family.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// The `{scope}` placeholder needs the padded and transformed scope string.
    #[must_use]
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = scope.into();
        self
    }

    /// The `{msg}` placeholder carries the actual log content — the most important part of every line.
    #[must_use]
    pub fn msg(mut self, msg: impl Into<String>) -> Self {
        self.msg = msg.into();
        self
    }

    /// File output needs timestamps for chronological analysis — terminal usually omits them.
    #[must_use]
    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = timestamp.into();
        self
    }

    /// Raw level name for templates that need it separately from the formatted tag.
    #[must_use]
    pub fn level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    /// Path templates use `{app}` to organize logs into per-application directories.
    #[must_use]
    pub fn app(mut self, app: impl Into<String>) -> Self {
        self.app = app.into();
        self
    }

    /// Path templates use `{year}/{month}/{day}` for date-based log directory hierarchies.
    #[must_use]
    pub fn date(mut self, year: &str, month: &str, day: &str) -> Self {
        self.year = year.to_string();
        self.month = month.to_string();
        self.day = day.to_string();
        self
    }
}
