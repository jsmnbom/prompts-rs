use prompts::{select::SelectPrompt, Prompt};

#[derive(Clone, Debug)] // Must derive Clone
struct Person {
    first_name: &'static str, // Static for example purposes
    last_name: &'static str,  // Static for example purposes
    id: usize,
}
impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.first_name, self.last_name)
    }
}

#[tokio::main]
async fn main() {
    // Build a list of people
    let data = vec![
        Person {
            first_name: "John",
            last_name: "Doe",
            id: 5,
        },
        Person {
            first_name: "Jane",
            last_name: "Doe",
            id: 3,
        },
        Person {
            first_name: "AN",
            last_name: "Other",
            id: 7,
        },
    ];
    // Prepare prompt
    let mut prompt = SelectPrompt::new("Choose a person", data);

    println!("Running prompt: {:?}", prompt);

    // Run the prompt and echo the chosen person's id
    match prompt.run().await {
        Ok(Some(person)) => println!("That persons's id is: {}", person.id),
        Ok(None) => println!("Prompt was aborted!"),
        Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
    }
}
