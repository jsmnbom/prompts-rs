//! Interactive prompt where the user chooses from a list of options

use crate::{
    utils::{
        calc_entries, is_abort_event, print_input_icon, print_state_icon, Figures, PromptState,
    },
    Prompt,
};
use async_trait::async_trait;
use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{style, Attribute, Color, Print, PrintStyledContent, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size as terminal_size, Clear, ClearType},
};
use futures::StreamExt;
use std::cmp;
use std::fmt;
use std::io::{stdout, Write};

/// Interactive prompt where the user chooses from a list of options
///
/// Shows a list of options. Use <kbd>up</kbd>/<kbd>down</kbd> to navigate
/// and <kbd>enter</kbd> to submit.
/// The data vector can have a custom type but it must implement
/// `std::fmt::Display` as well as `std::clone::Clone` and `std::marker::Send`.
///
/// See `prompts::autocomplete::AutoCompletePrompt` a similar prompt
/// but that allows the user to filter options.
///
/// # Examples
///
/// ```rust,ignore
/// use prompts::{Prompt, select::{SelectPrompt}};
///
/// let data = vec!["The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"];
/// let mut prompt = SelectPrompt::new("Choose a word", data);
///
/// match prompt.run().await {
///     Ok(Some(s)) => println!("Your choice is: {}", s),
///     Ok(None) => println!("Prompt was aborted!"),
///     Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
/// }
/// ```
pub struct SelectPrompt<T> {
    message: String,
    state: PromptState,
    choices: Vec<T>,
    current: usize,
    limit: usize,
}
impl<T: std::fmt::Debug> fmt::Debug for SelectPrompt<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SelectPrompt")
            .field("message", &self.message)
            .field("choices", &self.choices)
            .finish()
    }
}
impl<T: std::clone::Clone + std::marker::Send + std::fmt::Display> SelectPrompt<T> {
    /// Returns a SelectPrompt ready to be run
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display to the user before the prompt
    /// * `choices` - A vector of options that the user can choose from
    pub fn new<S>(message: S, choices: Vec<T>) -> SelectPrompt<T>
    where
        S: Into<String>,
    {
        SelectPrompt {
            message: message.into(),
            choices,
            state: PromptState::default(),
            current: 0,
            limit: 10,
        }
    }
}
#[async_trait]
impl<T: std::clone::Clone + std::marker::Send + std::fmt::Display> Prompt<T> for SelectPrompt<T> {
    /// Runs the prompt
    ///
    /// Stops either when the user selects an option, an error occurs,
    /// or the prompt is aborted by the user using CTRL+c, CTRL+z or ESC.
    async fn run(&mut self) -> std::result::Result<Option<T>, crossterm::ErrorKind> {
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
                    return Ok(Some(self.choices[self.current].clone()));
                }
                _ => (),
            }
        }
    }
    fn display(&mut self) -> crossterm::Result<()> {
        let mut stdout = stdout();

        let (start_index, end_index) = calc_entries(
            self.current,
            self.choices.len(),
            cmp::min(self.limit, (terminal_size()?.1 - 1) as usize),
        );

        if self.state == PromptState::Created {
            queue!(stdout, cursor::Hide)?;
            self.state = PromptState::Running;
        } else {
            queue!(
                stdout,
                cursor::MoveUp((end_index - start_index) as u16),
                cursor::MoveToColumn(0),
                Clear(ClearType::FromCursorDown)
            )?;
        }

        queue!(
            stdout,
            print_state_icon(&self.state),
            Print(" "),
            PrintStyledContent(style(&self.message).attribute(Attribute::Bold))
        )?;
        if !self.state.is_done() {
            for i in start_index..end_index {
                let choice = self.choices[i].to_string();
                let prefix = if i == start_index && start_index > 0 {
                    Figures::ArrowUp.as_str()
                } else if i == end_index - 1 && end_index < self.choices.len() {
                    Figures::ArrowDown.as_str()
                } else {
                    " "
                };
                queue!(
                    stdout,
                    Print("\n\r"),
                    PrintStyledContent(if i == self.current {
                        style(Figures::Pointer.as_str()).with(Color::Cyan)
                    } else {
                        style(" ")
                    }),
                    Print(format!(" {} ", prefix)),
                    PrintStyledContent(if i == self.current {
                        style(choice).attribute(Attribute::Bold).with(Color::Cyan)
                    } else {
                        style(choice)
                    }),
                )?;
            }
        }
        if self.state == PromptState::Success {
            queue!(
                stdout,
                Print(" "),
                print_input_icon(&self.state),
                Print(self.choices[self.current].to_string())
            )?;
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
                KeyCode::Enter => self.state = PromptState::Success,
                KeyCode::Home => {
                    self.current = 0;
                }
                KeyCode::End => {
                    self.current = self.choices.len() - 1;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.current = self.current.checked_sub(1).unwrap_or(0);
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.current = cmp::min(self.current + 1, self.choices.len() - 1);
                }
                _ => {}
            }
        }
    }
}
