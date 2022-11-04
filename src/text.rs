//! Interactive prompt that accepts text input

use crate::{
    utils::{is_abort_event, print_input_icon, print_state_icon, Figures, PromptState},
    Prompt,
};
use async_trait::async_trait;
use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{style, Attribute, Color, Print, PrintStyledContent, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use futures::StreamExt;
use std::cmp;
use std::fmt;
use std::io::{stdout, Write};

#[derive(Debug)]
pub enum Style {
    Normal,
    Password,
    Invisible,
}
impl Style {
    fn transform(&self, input: &str) -> String {
        match self {
            Style::Normal => return String::from(input),
            Style::Password => return (0..input.len()).map(|_| "*").collect::<String>(),
            Style::Invisible => return String::default(),
        }
    }
    fn cursor_mult(&self) -> usize {
        match self {
            Style::Normal => return 1,
            Style::Password => return 1,
            Style::Invisible => return 0,
        }
    }
}
impl Default for Style {
    fn default() -> Style {
        Style::Normal
    }
}

/// Interactive prompt that accepts text input
///
/// # Examples
///
/// ```
/// use prompt::{text::TextPrompt, Prompt};
/// let mut prompt = TextPrompt::new("What is your name?");
/// match prompt.run().await {
///     Ok(Some(s)) => println!("You wrote: {}", s),
///     Ok(None) => println!("Prompt was aborted!"),
///     Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
/// }
/// ```

#[derive(Default)]
pub struct TextPrompt {
    message: String,
    state: PromptState,
    input: String,
    cursor: usize,
    style: Style,
    validator: Option<fn(input: &str) -> std::result::Result<(), String>>,
    error: Option<String>,
}
impl fmt::Debug for TextPrompt {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("TextPrompt")
            .field("message", &self.message)
            .field("style", &self.style)
            .field(
                "validator",
                &format_args!(
                    "{}",
                    &match self.validator {
                        Some(_) => "custom validator",
                        None => "None",
                    }
                ),
            )
            .finish()
    }
}
impl TextPrompt {
    /// Returns a TextPrompt ready to be run
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display to the user before the prompt
    pub fn new<S>(message: S) -> TextPrompt
    where
        S: Into<String>,
    {
        TextPrompt {
            message: message.into(),
            ..Default::default()
        }
    }

    /// Mask typed letters with a given style (currently only invisible or password)
    ///
    /// # Arguments
    ///
    /// * `style` - The style to use
    pub fn with_style(mut self, style: Style) -> TextPrompt {
        self.style = style;
        self
    }

    /// Provide a custom validation closure
    ///
    /// # Arguments
    ///
    /// * `validator` - Validation closure that accepts a string, and returns
    ///     Ok if valid, or a string error to show if invalid.
    pub fn with_validator(
        mut self,
        validator: fn(input: &str) -> std::result::Result<(), String>,
    ) -> TextPrompt {
        self.validator = Some(validator);
        self
    }
}
#[async_trait]
impl Prompt<String> for TextPrompt {
    async fn run(&mut self) -> std::result::Result<Option<String>, crossterm::ErrorKind> {
        enable_raw_mode()?;
        let mut reader = EventStream::new();

        self.display()?;

        loop {
            match reader.next().await {
                Some(Ok(Event::Key(event))) => self.handle_key_event(event),
                Some(Err(e)) => {
                    disable_raw_mode()?;
                    return Err(e);
                }
                _ => {}
            }

            match self.state {
                PromptState::Validate => match self.validator {
                    Some(validator) => match validator(&self.input) {
                        Ok(()) => self.state = PromptState::Success,
                        Err(msg) => {
                            self.error = Some(msg);
                            self.state = PromptState::Running;
                        }
                    },
                    None => self.state = PromptState::Success,
                },
                _ => (),
            }

            self.display()?;

            match self.state {
                PromptState::Aborted => {
                    disable_raw_mode()?;
                    return Ok(None);
                }
                PromptState::Success => {
                    disable_raw_mode()?;
                    return Ok(Some(self.input.clone()));
                }
                _ => (),
            }
        }
    }
    fn display(&mut self) -> crossterm::Result<()> {
        let mut stdout = stdout();
        queue!(
            stdout,
            cursor::MoveToColumn(0),
            Clear(ClearType::FromCursorDown),
            print_state_icon(&self.state),
            Print(" "),
            PrintStyledContent(style(&self.message).attribute(Attribute::Bold)),
            Print(" "),
            print_input_icon(&self.state),
        )?;
        if !self.state.is_done() {
            let input_column =
                (2 + self.message.len() + 3 + (self.cursor * self.style.cursor_mult()) + 1) as u16;

            queue!(stdout, Print(self.style.transform(&self.input)),)?;
            match &self.error {
                None => queue!(stdout, cursor::MoveToColumn(input_column))?,
                Some(msg) => queue!(
                    stdout,
                    Print(format!("\n\r{} ", Figures::PointerSmall.as_str())),
                    PrintStyledContent(style(msg).with(Color::Red).attribute(Attribute::Italic)),
                    cursor::MoveToPreviousLine(1),
                    cursor::MoveToColumn(input_column)
                )?,
            }
        } else {
            queue!(stdout, Print("\n\r"))?;
        }
        stdout.flush()?;
        crossterm::Result::Ok(())
    }
    fn handle_key_event(&mut self, event: KeyEvent) {
        if is_abort_event(event) {
            self.state = PromptState::Aborted;
            return;
        }
        if event.modifiers == KeyModifiers::empty() {
            match event.code {
                KeyCode::Enter => self.state = PromptState::Validate,
                KeyCode::Backspace => {
                    self.cursor = self.cursor.checked_sub(1).unwrap_or(0);
                    if self.input.len() > self.cursor {
                        self.input.remove(self.cursor);
                    }
                }
                KeyCode::Left => {
                    self.cursor = self.cursor.checked_sub(1).unwrap_or(0);
                }
                KeyCode::Right => {
                    self.cursor = cmp::min(self.cursor + 1, self.input.len());
                }
                KeyCode::Home => {
                    self.cursor = 0;
                }
                KeyCode::End => {
                    self.cursor = self.input.len();
                }
                KeyCode::Char(c) => {
                    self.input.insert(self.cursor, c);
                    self.cursor += 1;
                }
                _ => {}
            }
        }
    }
}
