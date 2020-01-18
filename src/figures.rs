pub enum Figures {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    RadioOn,
    RadioOff,
    Tick,
    Cross,
    Ellipsis,
    PointerSmall,
    Line,
    Pointer,
}
impl Figures {
    #[cfg(windows)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Figures::ArrowUp => "↑",
            Figures::ArrowDown => "↓",
            Figures::ArrowLeft => "←",
            Figures::ArrowRight => "→",
            Figures::RadioOn => "(*)",
            Figures::RadioOff => "( )",
            Figures::Tick => "√",
            Figures::Cross => "×",
            Figures::Ellipsis => "...",
            Figures::PointerSmall => "»",
            Figures::Line => "─",
            Figures::Pointer => ">",
        }
    }
    #[cfg(not(windows))]
    pub fn as_str(&self) -> &'static str {
        match self {
            Figures::ArrowUp => "↑",
            Figures::ArrowDown => "↓",
            Figures::ArrowLeft => "←",
            Figures::ArrowRight => "→",
            Figures::RadioOn => "◉",
            Figures::RadioOff => "◯",
            Figures::Tick => "✔",
            Figures::Cross => "✖",
            Figures::Ellipsis => "…",
            Figures::PointerSmall => "›",
            Figures::Line => "─",
            Figures::Pointer => "❯",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_all() {
        let figures = [
            Figures::ArrowUp,
            Figures::ArrowDown,
            Figures::ArrowLeft,
            Figures::ArrowRight,
            Figures::RadioOn,
            Figures::RadioOff,
            Figures::Tick,
            Figures::Cross,
            Figures::Ellipsis,
            Figures::PointerSmall,
            Figures::Line,
            Figures::Pointer,
        ];
        for figure in figures.iter() {
            println!("{}", figure.as_str());
        }
    }
}
