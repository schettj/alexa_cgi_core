use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AlexaRequestPayload {
    pub version: String,
    pub session: Option<Session>,
    pub context: Option<Context>,
    pub request: RequestBody,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Session {
    #[serde(rename = "new")]
    pub is_new: bool,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub application: Application,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Context {
    #[serde(rename = "System")]
    pub system: Option<SystemContext>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SystemContext {
    pub application: Application,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Application {
    #[serde(rename = "applicationId")]
    pub application_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RequestBody {
    #[serde(rename = "type")]
    pub request_type: String,
    pub timestamp: String,
    pub intent: Option<Intent>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Intent {
    pub name: String,
}
