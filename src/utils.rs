//! Utility functions and enums for prompts
//!
//! Public in case you want to implement your own custom prompts

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::{style, Color, PrintStyledContent, Stylize},
};
use std::cmp;

/// Figures that are used for the prompts
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

/// Internal state of a prompt
#[derive(Eq, PartialEq, Debug)]
pub enum PromptState {
    /// Prompt was just created (and has not yet been displayed for the first time)
    Created,
    /// Prompt is running/displaying
    Running,
    /// Prompt input needs validation
    Validate,
    /// The prompt was aborted by the user
    Aborted,
    /// The prompt completed successfully
    Success,
}
impl Default for PromptState {
    fn default() -> PromptState {
        PromptState::Created
    }
}
impl PromptState {
    pub fn is_done(&self) -> bool {
        *self == PromptState::Aborted || *self == PromptState::Success
    }
}

/// Should we abort on this event
///
/// Returns true on CTRL+c, CTRL+z and ESC
pub fn is_abort_event(event: KeyEvent) -> bool {
    match event {
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('c'),
            ..
        } => true,
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('d'),
            ..
        } => true,
        KeyEvent {
            modifiers,
            code: KeyCode::Esc,
            ..
        } if modifiers == KeyModifiers::empty() => true,
        _ => false,
    }
}

/// Prints a cross, a tick or a question mark depending on prompt state
pub fn print_state_icon(state: &PromptState) -> PrintStyledContent<&'static str> {
    PrintStyledContent(match state {
        PromptState::Aborted => style(Figures::Cross.as_str()).with(Color::Red),
        PromptState::Success => style(Figures::Tick.as_str()).with(Color::Green),
        _ => style("?").with(Color::Magenta),
    })
}

/// Prints a pointer or ellipsis depending on prompt state
pub fn print_input_icon(state: &PromptState) -> PrintStyledContent<String> {
    PrintStyledContent(
        style(match state {
            PromptState::Aborted => "".to_string(),
            PromptState::Success => format!("{} ", Figures::Ellipsis.as_str()),
            _ => format!("{} ", Figures::PointerSmall.as_str()),
        })
        .with(Color::Grey),
    )
}

/// Returns start and end-index for showing a limited amount of items
///
/// Used for SelectPrompt and AutocompletePrompt
pub fn calc_entries(current: usize, total: usize, limit: usize) -> (usize, usize) {
    let start_index = cmp::min(
        total.checked_sub(limit).unwrap_or(0),
        current.checked_sub(limit / 2).unwrap_or(0),
    );
    let end_index = cmp::min(start_index + limit, total);
    (start_index, end_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
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
        let mut s = String::from("");
        for figure in figures.iter() {
            s.push_str(figure.as_str());
        }
        assert_eq!(s, "↑↓←→(*)( )√×...»─>");
    }

    #[test]
    #[cfg(not(windows))]
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
        let mut s = String::from("");
        for figure in figures.iter() {
            s.push_str(figure.as_str());
        }
        assert_eq!(s, "↑↓←→◉◯✔✖…›─❯");
    }
}
