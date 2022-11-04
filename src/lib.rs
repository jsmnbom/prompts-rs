pub mod autocomplete;
pub mod confirm;
pub mod multi_select;
pub mod select;
pub mod text;
pub mod utils;

use async_trait::async_trait;
use crossterm::event::KeyEvent;

/// Base prompt trait
///
/// You must `use` this when using any of the prompts in this crate
#[async_trait]
pub trait Prompt<T> {
    async fn run(&mut self) -> std::result::Result<Option<T>, crossterm::ErrorKind>;
    fn display(&mut self) -> crossterm::Result<()>;
    fn handle_key_event(&mut self, event: KeyEvent);
}
