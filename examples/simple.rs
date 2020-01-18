use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

use std::io::{stdout, Write};

use prompt::{Prompt, TextPromptType};

async fn test_prompts() {
    let mut p = Prompt::new(TextPromptType::new(), "What is your name?");
    match p.run().await {
        Ok(Some(s)) => write!(stdout(), "{}", s).unwrap(),
        Ok(None) => write!(stdout(), "abort!").unwrap(),
        Err(e) => write!(stdout(), "Error: {:?}", e).unwrap(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;

    test_prompts().await;

    disable_raw_mode()
}
