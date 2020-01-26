use prompt::{text::TextPrompt, Prompt};

#[tokio::main]
async fn main() {
    // Prepare the prompt
    let mut prompt = TextPrompt::new("What is your name?");

    // Run the prompt and echo the result
    match prompt.run().await {
        Ok(Some(s)) => println!("You wrote: {}", s),
        Ok(None) => println!("Prompt was aborted!"),
        Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
    }
}
