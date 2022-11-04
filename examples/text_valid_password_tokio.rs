use prompts::{
    text::{Style, TextPrompt},
    Prompt,
};

#[tokio::main]
async fn main() {
    // Prepare a prompt that asks for password between 1 and 20 characters (inclusive)
    let mut prompt = TextPrompt::new("What is your password?")
        .with_validator(|input| match input.len() {
            0 => Err("You must type something!".to_string()),
            1..=20 => Ok(()),
            _ => Err("You must not type more than 20 characters!".to_string()),
        })
        .with_style(Style::Password);

    println!("Running prompt: {:?}", prompt);

    // Run the prompt and echo the password
    match prompt.run().await {
        Ok(Some(s)) => println!("You wrote: {}", s),
        Ok(None) => println!("Prompt was aborted!"),
        Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
    }
}
