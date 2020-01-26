use prompt::{confirm::ConfirmPrompt, Prompt};

#[tokio::main]
async fn main() {
    // Prepare the prompt
    let mut prompt = ConfirmPrompt::new("Are you sure?").set_initial(true);

    // Run the prompt and echo the result
    match prompt.run().await {
        Ok(Some(true)) => println!("You were sure!"),
        Ok(Some(false)) => println!("You were not sure!"),
        Ok(None) => println!("Prompt was aborted!"),
        Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
    }
}
