use prompts::{select::SelectPrompt, text::TextPrompt, Prompt};

#[tokio::main]
async fn main() {
    // Prompt for first name and store it in var
    let mut first_name_prompt = TextPrompt::new("What is your first name?");
    let first_name = match first_name_prompt.run().await {
        Ok(Some(first_name)) => first_name,
        Ok(None) => panic!("Prompt was aborted!"),
        Err(e) => panic!("Some kind of crossterm error happened: {:?}", e),
    };

    // Prompt for last name and store it in var
    let mut last_name_prompt = TextPrompt::new("What is your last name?");
    let last_name = match last_name_prompt.run().await {
        Ok(Some(last_name)) => last_name,
        Ok(None) => panic!("Prompt was aborted!"),
        Err(e) => panic!("Some kind of crossterm error happened: {:?}", e),
    };

    // Prompt for place and store it in var
    let places = vec!["The north", "The south", "The west", "The east"];
    let mut place_prompt = SelectPrompt::new("Where are you from?", places);
    let from = match place_prompt.run().await {
        Ok(Some(from)) => from,
        Ok(None) => panic!("Prompt was aborted!"),
        Err(e) => panic!("Some kind of crossterm error happened: {:?}", e),
    };

    // Echo the details we collected
    println!("You are {} {} from {}!", first_name, last_name, from);
}
