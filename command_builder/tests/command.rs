use command_builder::*;

#[command("ping", "just a ping command")]
fn ping() {
    println!("Pong!");
}

#[command("hello", "hello world!")]
fn hello() {
    println!("Hello World!");
}

#[test]
pub fn test() {
    let input = "ping";
    command_tree!(input);
    commands!();
}
