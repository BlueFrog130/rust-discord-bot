use command_builder::{command, command_tree, commands};

#[command("ping", "Simple ping")]
fn ping() {
    println!("pong!");
}

pub fn commands<'a>() -> Vec<(&'a str, &'a str)> {
    Vec::from(commands!())
}

pub fn run(name: &str) {
    command_tree!(name);
}
