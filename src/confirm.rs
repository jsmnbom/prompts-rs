//! Interactive prompt where the user can choose yes or no

use crate::{
    utils::{is_abort_event, print_input_icon, print_state_icon, PromptState},
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
use std::fmt;
use std::io::{stdout, Write};

/// Interactive prompt where the user can choose yes or no
///
/// Use <kbd>y</kbd>/<kbd>n</kbd> to answer the prompt.
/// If default/initial is set <kbd>enter</kbd> will submit that value.
///
/// # Examples
///
/// ```rust,ignore
/// use prompts::{confirm::ConfirmPrompt, Prompt};
/// let mut prompt = ConfirmPrompt::new("Are you sure?");
///
/// match prompt.run().await {
///     Ok(Some(true)) => println!("You were sure!"),
///     Ok(Some(false)) => println!("You were not sure!"),
///     Ok(None) => println!("Prompt was aborted!"),
///     Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
/// }
/// ```
#[derive(Default)]
pub struct ConfirmPrompt {
    message: String,
    state: PromptState,
    answer: bool,
    initial: Option<bool>,
}
impl fmt::Debug for ConfirmPrompt {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ConfirmPrompt")
            .field("message", &self.message)
            .field("initial", &self.initial)
            .finish()
    }
}
impl ConfirmPrompt {
    /// Returns a ConfirmPrompt ready to be run
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display to the user before the prompt
    pub fn new<S>(message: S) -> ConfirmPrompt
    where
        S: Into<String>,
    {
        ConfirmPrompt {
            message: message.into(),
            ..Default::default()
        }
    }

    /// Set default/initial answer
    pub fn set_initial(mut self, initial: bool) -> ConfirmPrompt {
        self.initial = Some(initial);
        self
    }
}
#[async_trait]
impl Prompt<bool> for ConfirmPrompt {
    /// Runs the prompt
    ///
    /// Stops either when the user selects an option, an error occurs,
    /// or the prompt is aborted by the user using CTRL+c, CTRL+z or ESC.
    async fn run(&mut self) -> std::result::Result<Option<bool>, crossterm::ErrorKind> {
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

            self.display()?;

            match self.state {
                PromptState::Aborted => {
                    disable_raw_mode()?;
                    return Ok(None);
                }
                PromptState::Success => {
                    disable_raw_mode()?;
                    return Ok(Some(self.answer));
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
            queue!(
                stdout,
                PrintStyledContent(
                    style(match self.initial {
                        Some(true) => "(Y/n)",
                        Some(false) => "(y/N)",
                        None => "(y/n)",
                    })
                    .with(Color::DarkGrey)
                )
            )?;
        }
        if self.state == PromptState::Success {
            queue!(stdout, Print(if self.answer { "yes" } else { "no" }))?;
        }
        if self.state.is_done() {
            queue!(stdout, Print("\n\r"), cursor::Show)?;
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
                KeyCode::Enter => {
                    if let Some(initial) = self.initial {
                        self.answer = initial;
                        self.state = PromptState::Success;
                    }
                }
                KeyCode::Char('y') => {
                    self.answer = true;
                    self.state = PromptState::Success;
                }
                KeyCode::Char('n') => {
                    self.answer = false;
                    self.state = PromptState::Success;
                }
                _ => {}
            }
        }
    }
}
