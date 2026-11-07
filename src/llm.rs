//! Optional online AI mode — bring your own API key.
//!
//! The built-in local brain always stays available; online mode layers a real
//! LLM on top of it, and everything the user has taught ROCKET (facts,
//! corrections, preferences) is injected as the system prompt so the online
//! answers still feel like *their* ROCKET.

// Note: Using a simplified Brain interface for now
// use crate::ai::Brain;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Settings (persisted separately from the brain)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// Built-in local knowledge — free, instant, offline.
    Local,
    /// Bring-your-own-key LLM, trained on everything you taught ROCKET.
    Online,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    Groq,
    Gemini,
    OpenRouter,
    OpenAi,
    Cerebras,
    Mistral,
    DeepSeek,
    Together,
    Ollama,
    LMStudio,
    Custom,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl Provider {
    pub const ALL: &'static [Provider] = &[
        Provider::Groq,
        Provider::Cerebras,
        Provider::Gemini,
        Provider::Mistral,
        Provider::OpenRouter,
        Provider::DeepSeek,
        Provider::Together,
        Provider::OpenAi,
        Provider::Ollama,
        Provider::LMStudio,
        Provider::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Provider::Groq => "Groq (recommended)",
            Provider::Gemini => "Google Gemini",
            Provider::OpenRouter => "OpenRouter",
            Provider::OpenAi => "OpenAI (may block browsers)",
            Provider::Cerebras => "Cerebras (fast + free tier)",
            Provider::Mistral => "Mistral AI",
            Provider::DeepSeek => "DeepSeek",
            Provider::Together => "Together AI",
            Provider::Ollama => "Ollama (local, no key)",
            Provider::LMStudio => "LM Studio (local, no key)",
            Provider::Custom => "Custom (OpenAI-compatible URL)",
        }
    }

    /// OpenAI-compatible chat endpoint base (…/chat/completions is appended).
    pub fn default_base_url(self) -> &'static str {
        match self {
            Provider::Groq => "https://api.groq.com/openai/v1",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
            Provider::OpenRouter => "https://openrouter.ai/api/v1",
            Provider::OpenAi => "https://api.openai.com/v1",
            Provider::Cerebras => "https://api.cerebras.ai/v1",
            Provider::Mistral => "https://api.mistral.ai/v1",
            Provider::DeepSeek => "https://api.deepseek.com/v1",
            Provider::Together => "https://api.together.xyz/v1",
            Provider::Ollama => "http://localhost:11434/v1",
            Provider::LMStudio => "http://localhost:1234/v1",
            Provider::Custom => "",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Provider::Groq => "llama-3.3-70b-versatile",
            Provider::Gemini => "gemini-2.0-flash",
            Provider::OpenRouter => "openrouter/auto",
            Provider::OpenAi => "gpt-4o-mini",
            Provider::Cerebras => "llama-3.3-70b",
            Provider::Mistral => "mistral-small-latest",
            Provider::DeepSeek => "deepseek-chat",
            Provider::Together => "meta-llama/Llama-3.3-70B-Instruct-Turbo",
            Provider::Ollama => "llama3.2",
            Provider::LMStudio => "local-model",
            Provider::Custom => "",
        }
    }

    pub fn key_hint(self) -> &'static str {
        match self {
            Provider::Groq => "free key: console.groq.com/keys",
            Provider::Gemini => "free key: aistudio.google.com/apikey",
            Provider::OpenRouter => "key: openrouter.ai/keys",
            Provider::OpenAi => "key: platform.openai.com/api-keys",
            Provider::Cerebras => "free key: cloud.cerebras.ai",
            Provider::Mistral => "free tier key: console.mistral.ai",
            Provider::DeepSeek => "key: platform.deepseek.com",
            Provider::Together => "key: api.together.ai (settings → keys)",
            Provider::Ollama => "local & free — run `ollama serve`, then no key needed",
            Provider::LMStudio => "local & free — start the LM Studio server, no key needed",
            Provider::Custom => "paste your endpoint below (…/v1)",
        }
    }

    /// Local runtimes (Ollama / LM Studio) run without an API key.
    pub fn requires_key(self) -> bool {
        !matches!(self, Provider::Ollama | Provider::LMStudio)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub mode: Mode,
    pub provider: Provider,
    pub api_key: String,
    pub model: String,
    pub custom_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: Mode::Local,
            provider: Provider::Groq,
            api_key: String::new(),
            model: Provider::Groq.default_model().to_string(),
            custom_url: String::new(),
        }
    }
}

impl Settings {
    pub fn load() -> Settings {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_storage::{LocalStorage, Storage};
            if let Ok(s) = LocalStorage::get::<Settings>("rocket.llm.v1") {
                return s;
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(bytes) = std::fs::read("rocket-llm.json") {
                if let Ok(s) = serde_json::from_slice::<Settings>(&bytes) {
                    return s;
                }
            }
        }
        Settings::default()
    }

    pub fn save(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_storage::{LocalStorage, Storage};
            let _ = LocalStorage::set("rocket.llm.v1", self);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(bytes) = serde_json::to_vec(self) {
                let _ = std::fs::write("rocket-llm.json", bytes);
            }
        }
    }

    pub fn base_url(&self) -> String {
        if self.provider == Provider::Custom {
            self.custom_url.trim_end_matches('/').to_string()
        } else {
            self.provider.default_base_url().to_string()
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.base_url().is_empty()
            && (!self.provider.requires_key() || !self.api_key.trim().is_empty())
            && !self.model.trim().is_empty()
    }
}

/// Fire a tiny real request at the configured provider — the engine behind
/// the "🔌 Test connection" button. Falls back to the provider's default
/// model when none is set, so a fresh config can be tested as-is.
#[cfg(not(target_arch = "wasm32"))]
pub async fn ping(settings: Settings) -> Result<String, String> {
    let mut s = settings;
    if s.model.trim().is_empty() {
        s.model = s.provider.default_model().to_string();
    }
    let provider_label = s.provider.label().to_string();
    chat(
        s,
        "You are a connection test. Reply with the single word: pong".to_string(),
        vec![Turn {
            from_user: true,
            text: "ping".to_string(),
            image_data_url: None,
        }],
    )
    .await
    .map(|reply| format!("{} is alive — model replied: \"{}\"", provider_label, reply.trim()))
}

// ---------------------------------------------------------------------------
// System prompt — bakes the local brain into the online answers
// ---------------------------------------------------------------------------

pub fn system_prompt() -> String {
    String::from(
        "You are Rocket, a friendly personal gaming AI companion (a rocket robot). \
         You give clear, practical gaming help: mechanics, progression, tips, strategies, \
         builds and comparisons across games like Minecraft, Roblox, Terraria, Palworld, \
         Pokemon, GTA, Stardew Valley and Once Human.\n\
         Be direct and useful, like a knowledgeable friend. Keep answers reasonably short \
         (a few sentences to a short list) unless depth is asked for. It is okay to be \
         wrong occasionally; if corrected, acknowledge it plainly and remember the fix.\n",
    )
}

// ---------------------------------------------------------------------------
// Chat request (browser fetch — wasm only)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ApiMessage<'a>>,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    message: String,
}

#[derive(Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

/// One turn of conversation history for the request.
pub struct Turn {
    pub from_user: bool,
    pub text: String,
    /// Optional attached picture as a `data:` URL (for vision-capable models).
    pub image_data_url: Option<String>,
}

/// Minimal standard base64 encoder (avoids an extra dependency).
pub fn base64_encode(data: &[u8]) -> String {
    const TBL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TBL[((n >> 18) & 63) as usize] as char);
        out.push(TBL[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TBL[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TBL[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Sniff an image mime type from magic bytes (falls back to png).
pub fn sniff_image_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if bytes.starts_with(b"GIF8") {
        "image/gif"
    } else if bytes.len() > 12 && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/png"
    }
}

/// Calls an OpenAI-compatible /chat/completions endpoint from the browser.
#[cfg(target_arch = "wasm32")]
pub async fn chat(settings: Settings, system: String, history: Vec<Turn>) -> Result<String, String> {
    use gloo_net::http::Request;

    let mut messages = vec![ApiMessage {
        role: "system",
        content: &system,
    }];
    for turn in history.iter() {
        messages.push(ApiMessage {
            role: if turn.from_user { "user" } else { "assistant" },
            content: &turn.text,
        });
    }

    let body = ChatRequest {
        model: settings.model.trim(),
        messages,
        temperature: 0.7,
    };

    if settings.provider == Provider::Custom && settings.base_url().is_empty() {
        return Err("No custom URL set — open ⚙️ and paste an OpenAI-compatible endpoint.".into());
    }

    let url = format!("{}/chat/completions", settings.base_url());
    let response = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", settings.api_key.trim()))
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Could not build request: {:?}", e))?
        .send()
        .await
        .map_err(|e| {
            format!(
                "Network error — the provider may not allow browser calls (CORS). {:?}",
                e
            )
        })?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Could not read response: {:?}", e))?;

    if status != 200 {
        // Try to pull the provider's error message out of the JSON body.
        if let Ok(err) = serde_json::from_str::<ApiError>(&text) {
            return Err(format!("{} (HTTP {})", err.error.message, status));
        }
        return Err(format!("HTTP {} — check your key/model in ⚙️ settings", status));
    }

    let parsed: ChatResponse = serde_json::from_str(&text)
        .map_err(|e| format!("Unexpected response format: {:?}", e))?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "Empty response from the model".to_string())
}

/// Desktop implementation — same OpenAI-compatible wire format via reqwest.
#[cfg(not(target_arch = "wasm32"))]
pub async fn chat(settings: Settings, system: String, history: Vec<Turn>) -> Result<String, String> {
    use reqwest::StatusCode;

    if settings.provider == Provider::Custom && settings.base_url().is_empty() {
        return Err("No custom URL set — paste an OpenAI-compatible endpoint (…/v1).".into());
    }

    let mut messages: Vec<serde_json::Value> = vec![serde_json::json!({
        "role": "system",
        "content": system,
    })];
    for turn in history.iter() {
        let role = if turn.from_user { "user" } else { "assistant" };
        // Vision-capable models take content as text + image_url parts.
        let content = match &turn.image_data_url {
            Some(data_url) => serde_json::json!([
                { "type": "text", "text": turn.text },
                { "type": "image_url", "image_url": { "url": data_url } },
            ]),
            None => serde_json::json!(turn.text),
        };
        messages.push(serde_json::json!({ "role": role, "content": content }));
    }

    let url = format!("{}/chat/completions", settings.base_url());
    let payload = serde_json::json!({
        "model": settings.model.trim(),
        "temperature": 0.7,
        "messages": messages,
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key.trim()))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Network error — check your internet connection. {:?}", e))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Could not read response: {:?}", e))?;

    if status != StatusCode::OK {
        if let Ok(err) = serde_json::from_str::<ApiError>(&text) {
            return Err(format!("{} (HTTP {})", err.error.message, status));
        }
        return Err(format!("HTTP {} — check your key/model in settings", status));
    }

    let parsed: ChatResponse = serde_json::from_str(&text)
        .map_err(|e| format!("Unexpected response format: {:?}", e))?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "Empty response from the model".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_is_non_empty() {
        let prompt = system_prompt();
        assert!(prompt.len() > 20, "system prompt should carry real instructions");
    }

    #[test]
    fn provider_defaults_are_consistent() {
        for &p in Provider::ALL {
            if p != Provider::Custom {
                assert!(!p.default_base_url().is_empty());
                assert!(!p.default_model().is_empty());
            }
            assert!(!p.label().is_empty());
            assert!(!p.key_hint().is_empty());
        }
    }

    #[test]
    fn settings_default_is_local_mode() {
        let s = Settings::default();
        assert_eq!(s.mode, Mode::Local);
        assert!(!s.is_ready()); // no key by default
    }

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn image_mime_is_sniffed_from_magic_bytes() {
        let png = vec![0x89, b'P', b'N', b'G', 1, 2, 3];
        assert_eq!(sniff_image_mime(&png), "image/png");
        let jpeg = vec![0xFF, 0xD8, 0xFF, 9];
        assert_eq!(sniff_image_mime(&jpeg), "image/jpeg");
        assert_eq!(sniff_image_mime(&[1, 2, 3]), "image/png");
    }

    #[test]
    fn custom_url_is_used_when_ready() {
        let mut s = Settings::default();
        s.provider = Provider::Custom;
        s.custom_url = "https://example.com/v1/".into();
        s.api_key = "k".into();
        s.model = "m".into();
        assert!(s.is_ready());
        assert_eq!(s.base_url(), "https://example.com/v1");
    }
}
