use serde_json::{json, Value};

pub struct AlexaResponseBuilder {
    speech_text: String,
    should_end_session: bool,
}

impl AlexaResponseBuilder {
    pub fn new() -> Self {
        Self {
            speech_text: String::new(),
            should_end_session: true,
        }
    }

    pub fn speak(mut self, text: &str) -> Self {
        self.speech_text = text.to_string();
        self
    }

    pub fn should_end_session(mut self, end: bool) -> Self {
        self.should_end_session = end;
        self
    }

    pub fn build(self) -> Value {
        json!({
            "version": "1.0",
            "response": {
                "outputSpeech": {
                    "type": "PlainText",
                    "text": self.speech_text
                },
                "shouldEndSession": self.should_end_session
            }
        })
    }
}
