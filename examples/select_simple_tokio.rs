use prompts::{select::SelectPrompt, Prompt};

#[tokio::main]
async fn main() {
    let data = vec![
        "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog", "lorem", "ipsum",
        "dolar", "sit",
    ];

    // Prepare the prompt
    let mut prompt = SelectPrompt::new("Choose a word", data);

    println!("Running prompt: {:?}", prompt);

    // Run the prompt and echo the selection
    match prompt.run().await {
        Ok(Some(s)) => println!("Your choice is: {}", s),
        Ok(None) => println!("Prompt was aborted!"),
        Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
    }
}
