//! Interactive prompt where the user can select multiple options from a list

use crate::utils::contains_filter;
use crate::{
    utils::{
        calc_entries, is_abort_event, print_input_icon, print_state_icon, Figures, PromptState,
    },
    Prompt,
};
use async_trait::async_trait;
use crossterm::style::PrintStyledContent;
use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{style, Attribute, Color, Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size as terminal_size, Clear, ClearType},
};
use futures::StreamExt;
use std::cmp;
use std::fmt;
use std::io::{stdout, Write};

/// Interactive prompt where the user can select multiple options from a list
///
/// Shows a list of options. Use <kbd>up</kbd>/<kbd>down</kbd> to navigate
/// and <kbd>space</kbd> to select or deselect options. Type anything to filter the list.
/// Press <kbd>enter</kbd> to submit the selections.
/// The default filter will simply check the choices start with the input's .to_string().
/// The data vector can have a custom type but it must implement
/// `std::fmt::Display` as well as `std::clone::Clone` and `std::marker::Send`.
///
/// # Examples
///
/// ```rust,ignore
/// use prompts::{Prompt, multiselect::{MultiSelectPrompt}};
///
/// let data = vec!["cheese", "tomato", "ham", "chili", "mushroom"];
/// let mut prompt = MultiSelectPrompt::new("Select you fillings", data);
///
/// match prompt.run().await {
///     Ok(Some(Vec<s>)) => {
///         let choices = s.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(", ");
///         println!("Your choices are: {choices}")
///     },
///     Ok(None) => println!("Prompt was aborted!"),
///     Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
/// }
/// ```
pub struct MultiSelectPrompt<T: std::clone::Clone + std::marker::Send + std::fmt::Display> {
    message: String,
    state: PromptState,
    choices: Vec<T>,
    selected: Vec<bool>,
    current: usize,
    limit: usize,
    search_str: String,
    cursor: usize,
    filter: fn(input: &str, choice: &T) -> bool,
}

impl<T: std::fmt::Debug + std::clone::Clone + std::marker::Send + std::fmt::Display> fmt::Debug
    for MultiSelectPrompt<T>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("AutocompletePrompt")
            .field("message", &self.message)
            .field("choices", &self.choices)
            .field("filter", &format_args!("{}", "unknown filter"))
            .finish()
    }
}

impl<T: std::clone::Clone + std::marker::Send + std::fmt::Display> MultiSelectPrompt<T> {
    /// Returns a MultiSelectPrompt ready to be run
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display to the user before the prompt
    /// * `choices` - A vector of options that the user can choose from
    pub fn new<S>(message: S, choices: Vec<T>) -> MultiSelectPrompt<T>
    where
        S: Into<String>,
    {
        let selected = std::iter::repeat(false).take(choices.len()).collect();
        MultiSelectPrompt {
            message: message.into(),
            choices,
            state: PromptState::default(),
            selected,
            current: 0,
            limit: 10,
            search_str: "".to_string(),
            cursor: 0,
            filter: contains_filter,
        }
    }
}

#[async_trait]
impl<T: std::clone::Clone + std::marker::Send + std::fmt::Display + std::cmp::PartialEq>
    Prompt<Vec<T>> for MultiSelectPrompt<T>
{
    /// Runs the prompt
    ///
    /// Stops either when the user hits enter, an error occurs,
    /// or the prompt is aborted by the user using CTRL+c, CTRL+z or ESC.
    async fn run(&mut self) -> std::result::Result<Option<Vec<T>>, crossterm::ErrorKind> {
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
                    let results = self
                        .choices
                        .iter()
                        .zip(self.selected.iter())
                        .filter(|(_, &selected)| selected)
                        .map(|(item, _)| item.clone())
                        .collect();
                    return Ok(Some(results));
                }
                _ => (),
            }
        }
    }
    fn display(&mut self) -> crossterm::Result<()> {
        let mut stdout = stdout();

        let filtered_choices = self
            .choices
            .iter()
            .zip(self.selected.iter())
            .filter(|(item, _)| (self.filter)(self.search_str.as_str(), item))
            .collect::<Vec<(&T, &bool)>>();

        self.current = cmp::min(self.current, filtered_choices.len().saturating_sub(1));

        let (start_index, end_index) = calc_entries(
            self.current,
            filtered_choices.len(),
            cmp::min(self.limit, (terminal_size()?.1 - 1) as usize),
        );

        if self.state == PromptState::Created {
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
            let input_column = (2 + self.message.len() + 3 + self.cursor) as u16;

            queue!(
                stdout,
                Print(" "),
                print_input_icon(&self.state),
                Print(&self.search_str),
                cursor::SavePosition
            )?;
            if start_index == end_index {
                queue!(
                    stdout,
                    Print("\n\r"),
                    PrintStyledContent(style("Nothing matched your search").with(Color::DarkGrey)),
                )?;
            } else {
                for i in start_index..end_index {
                    let choice = filtered_choices[i];
                    let choice_str = choice.0.to_string();
                    let selected = *choice.1;
                    let prefix = if i == start_index && start_index > 0 {
                        Figures::ArrowUp.as_str()
                    } else if i == end_index - 1 && end_index < filtered_choices.len() {
                        Figures::ArrowDown.as_str()
                    } else {
                        " "
                    };
                    queue!(
                        stdout,
                        Print("\n\r"),
                        PrintStyledContent(if selected {
                            style(Figures::Tick.as_str()).with(Color::Yellow)
                        } else {
                            style(" ")
                        }),
                        Print(format!(" {} ", prefix)),
                        PrintStyledContent(if i == self.current {
                            style(choice_str)
                                .attribute(Attribute::Bold)
                                .with(Color::Cyan)
                        } else if selected {
                            style(choice_str).attribute(Attribute::Bold)
                        } else {
                            style(choice_str)
                        }),
                    )?;
                }
            }

            queue!(
                stdout,
                cursor::RestorePosition,
                cursor::MoveToColumn(input_column)
            )?;
        }
        if self.state.is_done() {
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
        let filtered_choices = self
            .choices
            .iter()
            .filter(|item| (self.filter)(self.search_str.as_str(), item))
            .collect::<Vec<&T>>();
        let last_pos = filtered_choices.len().saturating_sub(1);

        if event.modifiers == KeyModifiers::empty() {
            match event.code {
                KeyCode::Enter => self.state = PromptState::Success,
                KeyCode::Home => {
                    self.current = 0;
                }
                KeyCode::End => {
                    self.current = self.choices.len() - 1;
                }
                KeyCode::Up => {
                    self.current = self.current.checked_sub(1).unwrap_or(last_pos);
                }
                KeyCode::Down => {
                    self.current = (self.current + 1) % (last_pos + 1);
                }
                KeyCode::Backspace => {
                    self.cursor = self.cursor.saturating_sub(1);
                    if self.search_str.len() > self.cursor {
                        self.search_str.remove(self.cursor);
                    }
                }
                KeyCode::Left => {
                    self.cursor = self.cursor.saturating_sub(1);
                }
                KeyCode::Right => {
                    self.cursor = cmp::min(self.cursor + 1, self.search_str.len());
                }
                KeyCode::Char(' ') => {
                    if let Some(choice) = filtered_choices.get(self.current) {
                        let index = self.choices.iter().position(|c| c == *choice);
                        if let Some(i) = index {
                            self.selected[i] = !self.selected[i];
                        }
                    }
                }
                KeyCode::Char(c) => {
                    self.search_str.insert(self.cursor, c);
                    self.cursor += 1;
                }
                _ => {}
            }
        }
    }
}
