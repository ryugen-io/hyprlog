//! Structure template parsing for log output.
//!
//! Templates use placeholders like `{tag}`, `{scope}`, `{msg}`.

/// Available placeholders in format strings.
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
    /// Returns the placeholder string (without braces).
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

    /// All available placeholders.
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

/// A parsed segment of a format string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatSegment {
    /// Literal text.
    Literal(String),
    /// A placeholder to be replaced.
    Placeholder(Placeholder),
}

/// A parsed format template.
#[derive(Debug, Clone)]
pub struct FormatTemplate {
    segments: Vec<FormatSegment>,
}

impl FormatTemplate {
    /// Parses a format string into a template.
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

    /// Returns the parsed segments.
    #[must_use]
    pub fn segments(&self) -> &[FormatSegment] {
        &self.segments
    }

    /// Renders the template with provided values.
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

/// Values to substitute into a format template.
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
    /// Creates new empty format values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tag value.
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Sets the icon value.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Sets the scope value.
    #[must_use]
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = scope.into();
        self
    }

    /// Sets the message value.
    #[must_use]
    pub fn msg(mut self, msg: impl Into<String>) -> Self {
        self.msg = msg.into();
        self
    }

    /// Sets the timestamp value.
    #[must_use]
    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = timestamp.into();
        self
    }

    /// Sets the level value.
    #[must_use]
    pub fn level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    /// Sets the app value.
    #[must_use]
    pub fn app(mut self, app: impl Into<String>) -> Self {
        self.app = app.into();
        self
    }

    /// Sets date values from year, month, day.
    #[must_use]
    pub fn date(mut self, year: &str, month: &str, day: &str) -> Self {
        self.year = year.to_string();
        self.month = month.to_string();
        self.day = day.to_string();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let template = FormatTemplate::parse("{tag} {msg}");
        assert_eq!(template.segments.len(), 3);
        assert_eq!(
            template.segments[0],
            FormatSegment::Placeholder(Placeholder::Tag)
        );
        assert_eq!(
            template.segments[1],
            FormatSegment::Literal(" ".to_string())
        );
        assert_eq!(
            template.segments[2],
            FormatSegment::Placeholder(Placeholder::Msg)
        );
    }

    #[test]
    fn parse_with_literals() {
        let template = FormatTemplate::parse("[{level}] {msg}");
        assert_eq!(template.segments.len(), 4);
        assert_eq!(
            template.segments[0],
            FormatSegment::Literal("[".to_string())
        );
    }

    #[test]
    fn render_template() {
        let template = FormatTemplate::parse("{tag} {scope}  {msg}");
        let values = FormatValues::new().tag("[INFO]").scope("MAIN").msg("hello");

        let result = template.render(&values);
        assert_eq!(result, "[INFO] MAIN  hello");
    }

    #[test]
    fn unknown_placeholder() {
        let template = FormatTemplate::parse("{unknown}");
        assert_eq!(
            template.segments[0],
            FormatSegment::Literal("{unknown}".to_string())
        );
    }

    #[test]
    fn path_template() {
        let template = FormatTemplate::parse("{year}/{month}/{app}");
        let values = FormatValues::new().date("2024", "01", "15").app("myapp");

        let result = template.render(&values);
        assert_eq!(result, "2024/01/myapp");
    }
}
