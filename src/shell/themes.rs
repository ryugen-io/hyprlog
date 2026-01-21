//! Prompt themes for the interactive shell.

use std::fmt::Write;
use std::str::FromStr;

/// RGB color tuple.
type Rgb = (u8, u8, u8);

/// All available themes.
pub const ALL_THEMES: &[Theme] = &[
    Theme::Dracula,
    Theme::Nord,
    Theme::Gruvbox,
    Theme::Catppuccin,
    Theme::TokyoNight,
    Theme::Synthwave,
    Theme::Matrix,
    Theme::Ocean,
];

/// Available prompt themes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dracula,
    Nord,
    Gruvbox,
    Catppuccin,
    TokyoNight,
    Synthwave,
    Matrix,
    Ocean,
}

impl Theme {
    /// Returns the theme name as a string.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dracula => "dracula",
            Self::Nord => "nord",
            Self::Gruvbox => "gruvbox",
            Self::Catppuccin => "catppuccin",
            Self::TokyoNight => "tokyonight",
            Self::Synthwave => "synthwave",
            Self::Matrix => "matrix",
            Self::Ocean => "ocean",
        }
    }

    /// Returns all available theme names.
    #[must_use]
    pub fn list() -> Vec<&'static str> {
        ALL_THEMES.iter().map(|t| t.name()).collect()
    }
    /// Returns the gradient colors for each character in "hyprlog>".
    #[must_use]
    pub const fn gradient(self) -> &'static [Rgb; 8] {
        match self {
            Self::Dracula => &[
                (255, 85, 85),   // red
                (255, 184, 108), // orange
                (241, 250, 140), // yellow
                (80, 250, 123),  // green
                (139, 233, 253), // cyan
                (189, 147, 249), // purple
                (255, 121, 198), // pink
                (255, 85, 85),   // red
            ],
            Self::Nord => &[
                (191, 97, 106),  // aurora red
                (208, 135, 112), // aurora orange
                (235, 203, 139), // aurora yellow
                (163, 190, 140), // aurora green
                (136, 192, 208), // frost
                (129, 161, 193), // frost
                (94, 129, 172),  // frost
                (180, 142, 173), // aurora purple
            ],
            Self::Gruvbox => &[
                (251, 73, 52),   // red
                (254, 128, 25),  // orange
                (250, 189, 47),  // yellow
                (184, 187, 38),  // green
                (142, 192, 124), // aqua
                (131, 165, 152), // blue
                (211, 134, 155), // purple
                (251, 73, 52),   // red
            ],
            Self::Catppuccin => &[
                (243, 139, 168), // red (mocha)
                (250, 179, 135), // peach
                (249, 226, 175), // yellow
                (166, 227, 161), // green
                (148, 226, 213), // teal
                (137, 180, 250), // blue
                (203, 166, 247), // mauve
                (245, 194, 231), // pink
            ],
            Self::TokyoNight => &[
                (247, 118, 142), // red
                (255, 158, 100), // orange
                (224, 175, 104), // yellow
                (158, 206, 106), // green
                (115, 218, 202), // teal
                (122, 162, 247), // blue
                (187, 154, 247), // purple
                (255, 0, 124),   // magenta
            ],
            Self::Synthwave => &[
                (255, 0, 128),   // hot pink
                (255, 56, 100),  // pink
                (255, 113, 206), // light pink
                (1, 205, 254),   // cyan
                (5, 217, 232),   // light cyan
                (185, 103, 255), // purple
                (255, 0, 255),   // magenta
                (255, 0, 128),   // hot pink
            ],
            Self::Matrix => &[
                (0, 255, 65), // bright green
                (0, 230, 60), // green
                (0, 205, 55), // green
                (0, 180, 50), // green
                (0, 155, 45), // darker green
                (0, 180, 50), // green
                (0, 205, 55), // green
                (0, 255, 65), // bright green
            ],
            Self::Ocean => &[
                (0, 119, 182),   // deep blue
                (0, 150, 199),   // blue
                (0, 180, 216),   // light blue
                (72, 202, 228),  // cyan
                (144, 224, 239), // light cyan
                (173, 232, 244), // pale cyan
                (202, 240, 248), // very pale
                (0, 119, 182),   // deep blue
            ],
        }
    }

    /// Builds the colored prompt string.
    #[must_use]
    pub fn build_prompt(self) -> String {
        let chars = ['h', 'y', 'p', 'r', 'l', 'o', 'g', '>'];
        let gradient = self.gradient();
        let mut prompt = String::new();

        for (i, c) in chars.iter().enumerate() {
            let (r, g, b) = gradient[i];
            let _ = write!(prompt, "\x1b[38;2;{r};{g};{b}m{c}");
        }
        prompt.push_str("\x1b[0m "); // reset + space
        prompt
    }
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dracula" => Ok(Self::Dracula),
            "nord" => Ok(Self::Nord),
            "gruvbox" => Ok(Self::Gruvbox),
            "catppuccin" => Ok(Self::Catppuccin),
            "tokyonight" | "tokyo-night" | "tokyo_night" => Ok(Self::TokyoNight),
            "synthwave" => Ok(Self::Synthwave),
            "matrix" => Ok(Self::Matrix),
            "ocean" => Ok(Self::Ocean),
            _ => Err(format!("Unknown theme: {s}")),
        }
    }
}
