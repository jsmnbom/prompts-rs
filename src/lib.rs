mod figures;

use std::io::{stdout, Write};

use std::cmp;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::{Attribute, Colorize, Print},
    terminal::{Clear, ClearType},
    Result,
};

use futures::{future::FutureExt, select, StreamExt};

#[derive(Eq, PartialEq, Debug)]
pub enum PromptState {
    Created,
    Running,
    Aborted,
    Done,
}

pub trait PromptType {
    fn display(&mut self, state: &PromptState, message: &String) -> Result<()>;
    fn output(&mut self) -> String;
    fn handle_event(&mut self, state: &PromptState, event: Event) -> Option<PromptState>;
}

pub struct Prompt<T>
where
    T: PromptType,
{
    prompt_type: T,
    message: String,
    state: PromptState,
}
impl<T> Prompt<T>
where
    T: PromptType,
{
    pub fn new<S>(prompt_type: T, message: S) -> Prompt<T>
    where
        S: Into<String>,
    {
        Prompt {
            prompt_type,
            message: message.into(),
            state: PromptState::Created,
        }
    }

    pub async fn run(&mut self) -> std::result::Result<Option<String>, crossterm::ErrorKind> {
        let mut reader = EventStream::new();

        self.prompt_type.display(&self.state, &self.message)?;

        loop {
            let mut event = reader.next().fuse();

            select! {
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            if let Some(new_state) = self.prompt_type.handle_event(&self.state, event) {
                                self.state = new_state;
                            }
                        },
                        Some(Err(e)) => return Err(e),
                        None => ()
                    }
                }
            };

            self.prompt_type.display(&self.state, &self.message)?;

            match self.state {
                PromptState::Created | PromptState::Running => (),
                PromptState::Aborted => return Ok(None),
                PromptState::Done => return Ok(Some(self.prompt_type.output())),
            }
        }
    }
}

#[derive(Default)]
pub struct TextPromptType {
    input: String,
    cursor: usize,
}

impl TextPromptType {
    pub fn new() -> TextPromptType {
        TextPromptType::default()
    }
}

impl PromptType for TextPromptType {
    fn display(&mut self, state: &PromptState, message: &String) -> Result<()> {
        let mut stdout = stdout();
        queue!(
            stdout,
            Clear(ClearType::CurrentLine),
            cursor::MoveToColumn(0),
            Print(match *state {
                PromptState::Aborted => figures::Figures::Cross.as_str().red(),
                PromptState::Done => figures::Figures::Tick.as_str().green(),
                _ => "?".magenta(),
            }),
            Print(" "),
            Print(Attribute::Bold),
            Print(message),
            Print(Attribute::Reset),
            Print(" "),
            Print(&self.input),
            cursor::MoveToColumn((2 + message.len() + 1 + self.cursor + 1) as u16),
        )?;
        if *state == PromptState::Done {
            queue!(stdout, Print("\n\r"))?;
        }
        stdout.flush()?;
        return Result::Ok(());
    }

    fn output(&mut self) -> String {
        self.input.clone()
    }

    fn handle_event(&mut self, state: &PromptState, event: Event) -> Option<PromptState> {
        let empty = KeyModifiers::empty();

        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('c'),
            }) => return Some(PromptState::Aborted),
            Event::Key(KeyEvent { modifiers, code }) if modifiers == empty => match code {
                KeyCode::Esc => return Some(PromptState::Aborted),
                KeyCode::Enter => return Some(PromptState::Done),
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
                // KeyCode::Left => execute!(stdout(), cursor::MoveLeft(1))?,
                // KeyCode::Right => execute!(stdout(), cursor::MoveRight(1))?,
                KeyCode::Char(c) => {
                    self.input.insert(self.cursor, c);
                    self.cursor += 1;
                }
                _ => {}
            },
            _ => {}
        }
        return None;
    }
}
