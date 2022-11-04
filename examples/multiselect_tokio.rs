use prompts::{multi_select::MultiSelectPrompt, Prompt};

#[tokio::main]
async fn main() {
    let data = vec![
        "cheese",
        "tomato",
        "ham",
        "mushroom",
        "pineapple",
        "pepperoni",
        "onion",
        "olive",
        "salami",
    ];

    // Prepare the prompt
    let mut prompt = MultiSelectPrompt::new("Choose your toppings", data);

    println!("Running prompt: {:?}", prompt);

    // Run the prompt and echo the selection
    match prompt.run().await {
        Ok(Some(s)) => {
            let choices: Vec<String> = s.into_iter().map(String::from).collect();
            println!("Your choices are: {}", choices.join(", "));
        }
        Ok(None) => println!("Prompt was aborted!"),
        Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
    }
}
