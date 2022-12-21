use bot_commands::commands;
use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Discord Token
    #[arg(short, long, env = "DISCORD_TOKEN")]
    token: String,

    /// Optional application Id to register commands on
    #[arg(short, long, env = "DISCORD_APPLICATION_ID")]
    application_id: String,

    #[arg(short, long, env = "DISCORD_GUILD_ID")]
    guild_id: String,
}

#[derive(Serialize)]
struct Command {
    name: String,
    description: String,
}

fn main() {
    let cli = Args::parse();
    let raw_commands = commands();
    let cmds: Vec<Command> = raw_commands
        .iter()
        .map(|c| Command {
            name: c.0.to_string(),
            description: c.1.to_string(),
        })
        .collect();

    let client = reqwest::blocking::Client::new();
    let res = client
        .put(format!(
            "https://discord.com/api/v10/applications/{}/guilds/{}/commands",
            cli.application_id, cli.guild_id
        ))
        .header(reqwest::header::AUTHORIZATION, format!("Bot {}", cli.token))
        .json(&cmds)
        .send();

    match res {
        Ok(response) => {
            println!("Registered commands");
            println!(
                "{}",
                response.text().unwrap_or(String::from("No response body"))
            );
        }
        Err(e) => {
            println!("Error registering commands");
            println!("{}", e.to_string());
        }
    }
}
