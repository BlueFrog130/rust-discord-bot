use discord::*;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use worker::*;

mod discord;
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(mut req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Only accepting POST
    if !matches!(req.method(), Method::Post) {
        return Response::error("Method not allowed", 405);
    }

    let public_key_string = env.secret("DISCORD_PUBLIC_KEY")?.to_string();
    let public_key_bytes = hex::decode(public_key_string).unwrap();

    let signature_string = req
        .headers()
        .get("X-Signature-Ed25519")?
        .expect("Missing Ed25519");

    let signature_bytes = hex::decode(signature_string).unwrap();

    let timestamp = req
        .headers()
        .get("X-Signature-Timestamp")?
        .expect("Missing Timestamp");

    let body = req.text().await?;

    let message = format!("{timestamp}{body}");

    let public_key = PublicKey::from_bytes(&public_key_bytes.as_slice()).unwrap();

    let signature = Signature::from_bytes(&signature_bytes.as_slice()).unwrap();

    let verify_result = public_key.verify(message.as_bytes(), &signature);

    if let Err(_) = verify_result {
        return Response::error("", 401);
    }

    console_debug!("{}", body.as_str());

    let interaction: Interaction = serde_json::from_str(body.as_str())?;

    console_debug!("{:?}", interaction);

    match interaction {
        Interaction::Ping => {
            console_log!("{} - Received PING", Date::now().to_string());
            let response = InteractionResponse::Pong;
            Response::from_json(&response)
        }
        Interaction::ApplicationCommand { data, interaction } => {
            console_log!(
                "{} - [{} | {} | {}] {}",
                Date::now().to_string(),
                interaction.application_id,
                interaction.guild_id.unwrap_or("No guild".to_string()),
                interaction.channel_id.unwrap_or("No channel".to_string()),
                data.name
            );
            match data.name.as_str() {
                "ping" => {
                    let response = InteractionResponse::ChannelMessageWithSource(
                        ChannelMessageWithSourceData::content("Pong!"),
                    );

                    Response::from_json(&response)
                }
                _ => Response::error("Unknown command", 400),
            }
        }
    }
}
