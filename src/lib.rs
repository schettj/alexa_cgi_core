pub mod request;
pub mod response;
pub mod verifier;

use std::io::{self, Read};
use serde_json::Value;

pub use request::AlexaRequestPayload;
pub use response::AlexaResponseBuilder;
pub use verifier::RequestVerifier;

/// The core trait clients implement to define custom skill intentions
pub trait AlexaSkill {
    fn skill_id(&self) -> Option<&str> { None }
    
    fn handle_launch(&self) -> String;
    fn handle_intent(&self, intent_name: &str) -> String;
    fn handle_session_ended(&self) {}
    
    fn handle_fallback(&self) -> String {
        String::from("I didn't quite catch that. Please try again.")
    }
}

/// The generic driver engine executing the CGI runtime process loop
pub fn run_cgi_skill<T: AlexaSkill>(skill: T) {
    let request_method = std::env::var("REQUEST_METHOD").unwrap_or_default();
    let content_length: usize = std::env::var("CONTENT_LENGTH")
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);

    if request_method != "POST" {
        println!("Status: 405 Method Not Allowed\nContent-Type: text/plain\n\nPOST required.");
        return;
    }

    let mut buffer = vec![0; content_length];
    if io::stdin().read_exact(&mut buffer).is_err() {
        print_error_response("Failed to read raw stdin payload stream.");
        return;
    }

    let payload: AlexaRequestPayload = match serde_json::from_slice(&buffer) {
        Ok(p) => p,
        Err(_) => {
            print_error_response("Invalid JSON payload structure.");
            return;
        }
    };

    // Instantiate safety verifications
    let verifier = RequestVerifier::new(skill.skill_id());
    if let Err(err_msg) = verifier.verify(&payload) {
        println!("Status: 400 Bad Request\nContent-Type: text/plain\n\n{}", err_msg);
        return;
    }

    // Process intent execution vectors matching our custom trait definitions
    match payload.request.request_type.as_str() {
        "LaunchRequest" => {
            let text = skill.handle_launch();
            print_success_response(AlexaResponseBuilder::new().speak(&text).build());
        }
        "IntentRequest" => {
            let intent_name = payload.request.intent.as_ref().map(|i| i.name.as_str()).unwrap_or("");
            let text = match intent_name {
                "AMAZON.StopIntent" | "AMAZON.CancelIntent" => String::from("Goodbye!"),
                "AMAZON.HelpIntent" | "AMAZON.FallbackIntent" => skill.handle_fallback(),
                name => skill.handle_intent(name),
            };
            print_success_response(AlexaResponseBuilder::new().speak(&text).build());
        }
        "SessionEndedRequest" => {
            skill.handle_session_ended();
            println!("Status: 200 OK\n\n");
        }
        _ => {
            print_error_response("Unsupported request frame type.");
        }
    }
}

fn print_success_response(json_val: Value) {
    if let Ok(out_string) = serde_json::to_string(&json_val) {
        println!("Status: 200 OK");
        println!("Content-Type: application/json;charset=UTF-8");
        println!("Content-Length: {}", out_string.len());
        println!();
        print!("{}", out_string);
    }
}

fn print_error_response(msg: &str) {
    let json_val = AlexaResponseBuilder::new().speak(msg).build();
    print_success_response(json_val);
}
