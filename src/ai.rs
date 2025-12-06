use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;

/// AI platform types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum AIPlatform {
    OpenAI,
    Anthropic,
    GoogleGemini,
    Mistral,
    Ollama,
}

/// AI model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AIModel {
    platform: AIPlatform,
    model_name: String,
    api_key: String,
    base_url: Option<String>,
}

/// AI prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromptTemplate {
    name: String,
    template: String,
    platform: Option<AIPlatform>,
}

/// AI response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    content: String,
    model: String,
    platform: AIPlatform,
    tokens_used: Option<usize>,
}

impl AIResponse {
    /// Get the content of the response
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the model used for the response
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the platform used for the response
    pub fn platform(&self) -> AIPlatform {
        self.platform
    }

    /// Get the tokens used for the response
    pub fn tokens_used(&self) -> Option<usize> {
        self.tokens_used
    }
}

/// AI client for interacting with multiple AI platforms
#[derive(Clone)]
pub struct AIClient {
    client: Arc<Client>,
    models: HashMap<String, AIModel>,
    templates: HashMap<String, PromptTemplate>,
    default_model: String,
}

impl Default for AIClient {
    fn default() -> Self {
        Self::new().expect("Failed to create AI client")
    }
}

impl AIClient {
    /// Create a new AI client with default configuration
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let client = Arc::new(Client::new());
        let mut models = HashMap::new();
        let mut templates = HashMap::new();

        // Load environment variables for default models
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            models.insert(
                "openai-gpt4".to_string(),
                AIModel {
                    platform: AIPlatform::OpenAI,
                    model_name: "gpt-4".to_string(),
                    api_key,
                    base_url: None,
                },
            );
        }

        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            models.insert(
                "anthropic-claude3".to_string(),
                AIModel {
                    platform: AIPlatform::Anthropic,
                    model_name: "claude-3-opus-20240229".to_string(),
                    api_key,
                    base_url: None,
                },
            );
        }

        if let Ok(api_key) = env::var("GOOGLE_API_KEY") {
            models.insert(
                "google-gemini".to_string(),
                AIModel {
                    platform: AIPlatform::GoogleGemini,
                    model_name: "gemini-pro".to_string(),
                    api_key,
                    base_url: None,
                },
            );
        }

        if let Ok(api_key) = env::var("MISTRAL_API_KEY") {
            models.insert(
                "mistral-large".to_string(),
                AIModel {
                    platform: AIPlatform::Mistral,
                    model_name: "mistral-large-latest".to_string(),
                    api_key,
                    base_url: None,
                },
            );
        }

        // Default model
        let default_model = models
            .keys()
            .next()
            .unwrap_or(&"ollama-llama3".to_string())
            .clone();

        // Add default Ollama model (no API key needed)
        models
            .entry("ollama-llama3".to_string())
            .or_insert(AIModel {
                platform: AIPlatform::Ollama,
                model_name: "llama3".to_string(),
                api_key: "".to_string(),
                base_url: Some("http://localhost:11434".to_string()),
            });

        // Load default templates
        templates.insert(
            "code-generation".to_string(),
            PromptTemplate {
                name: "code-generation".to_string(),
                template: "Generate {language} code for the following task: {prompt}\n\nProvide only the code, no explanations.".to_string(),
                platform: None,
            },
        );

        templates.insert(
            "code-explanation".to_string(),
            PromptTemplate {
                name: "code-explanation".to_string(),
                template: "Explain the following {language} code:\n\n{code}\n\nProvide a clear, concise explanation.".to_string(),
                platform: None,
            },
        );

        templates.insert(
            "code-optimization".to_string(),
            PromptTemplate {
                name: "code-optimization".to_string(),
                template: "Optimize the following {language} code for performance and readability:\n\n{code}\n\nExplain the optimizations made.".to_string(),
                platform: None,
            },
        );

        Ok(Self {
            client,
            models,
            templates,
            default_model,
        })
    }

    /// Generate code using AI
    pub async fn generate_code(
        &mut self,
        prompt: &str,
        language: Option<&str>,
    ) -> Result<String, Box<dyn Error>> {
        let language = language.unwrap_or("rust");

        // Format prompt using template
        let template = self.templates.get("code-generation").unwrap();
        let formatted_prompt = template
            .template
            .replace("{language}", language)
            .replace("{prompt}", prompt);

        // Generate response using default model
        let response = self.generate_response(&formatted_prompt, None).await?;

        Ok(response.content)
    }

    /// Generate response from AI
    pub async fn generate_response(
        &self,
        prompt: &str,
        model_name: Option<&str>,
    ) -> Result<AIResponse, Box<dyn Error>> {
        let model_name = model_name.unwrap_or(&self.default_model);
        let model = self
            .models
            .get(model_name)
            .ok_or(format!("Model '{}' not found", model_name))?;

        match model.platform {
            AIPlatform::OpenAI => self.generate_openai_response(model, prompt).await,
            AIPlatform::Anthropic => self.generate_anthropic_response(model, prompt).await,
            AIPlatform::GoogleGemini => self.generate_google_gemini_response(model, prompt).await,
            AIPlatform::Mistral => self.generate_mistral_response(model, prompt).await,
            AIPlatform::Ollama => self.generate_ollama_response(model, prompt).await,
        }
    }

    /// Generate response from OpenAI
    async fn generate_openai_response(
        &self,
        model: &AIModel,
        prompt: &str,
    ) -> Result<AIResponse, Box<dyn Error>> {
        // OpenAI API request
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<OpenAIMessage>,
            temperature: f32,
        }

        #[derive(Serialize, Deserialize)]
        struct OpenAIMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<OpenAIChoice>,
            usage: Option<OpenAIUsage>,
        }

        #[derive(Deserialize)]
        struct OpenAIChoice {
            message: OpenAIMessage,
        }

        #[derive(Deserialize)]
        struct OpenAIUsage {
            total_tokens: usize,
        }

        let request = OpenAIRequest {
            model: model.model_name.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
        };

        let base_url = model
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com");
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<OpenAIResponse>()
            .await?;

        let choice = response.choices.first().ok_or("No choices in response")?;

        Ok(AIResponse {
            content: choice.message.content.clone(),
            model: model.model_name.clone(),
            platform: AIPlatform::OpenAI,
            tokens_used: response.usage.map(|u| u.total_tokens),
        })
    }

    /// Generate response from Anthropic
    async fn generate_anthropic_response(
        &self,
        model: &AIModel,
        prompt: &str,
    ) -> Result<AIResponse, Box<dyn Error>> {
        // Anthropic API request
        #[derive(Serialize)]
        struct AnthropicRequest {
            model: String,
            messages: Vec<AnthropicMessage>,
            max_tokens: usize,
            temperature: f32,
        }

        #[derive(Serialize)]
        struct AnthropicMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct AnthropicResponse {
            content: Vec<AnthropicContent>,
            usage: AnthropicUsage,
        }

        #[derive(Deserialize)]
        struct AnthropicContent {
            text: String,
        }

        #[derive(Deserialize)]
        struct AnthropicUsage {
            input_tokens: usize,
            output_tokens: usize,
        }

        let request = AnthropicRequest {
            model: model.model_name.clone(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 2048,
            temperature: 0.7,
        };

        let base_url = model
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com");
        let response = self
            .client
            .post(format!("{}/v1/messages", base_url))
            .header("x-api-key", &model.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?
            .json::<AnthropicResponse>()
            .await?;

        let content = response.content.first().ok_or("No content in response")?;

        Ok(AIResponse {
            content: content.text.clone(),
            model: model.model_name.clone(),
            platform: AIPlatform::Anthropic,
            tokens_used: Some(response.usage.input_tokens + response.usage.output_tokens),
        })
    }

    /// Generate response from Google Gemini
    async fn generate_google_gemini_response(
        &self,
        model: &AIModel,
        prompt: &str,
    ) -> Result<AIResponse, Box<dyn Error>> {
        // Google Gemini API request
        #[derive(Serialize)]
        struct GoogleGeminiRequest {
            contents: Vec<GoogleGeminiContent>,
        }

        #[derive(Serialize, Deserialize)]
        struct GoogleGeminiContent {
            parts: Vec<GoogleGeminiPart>,
        }

        #[derive(Serialize, Deserialize)]
        struct GoogleGeminiPart {
            text: String,
        }

        #[derive(Deserialize)]
        struct GoogleGeminiResponse {
            candidates: Vec<GoogleGeminiCandidate>,
            usage_metadata: GoogleGeminiUsage,
        }

        #[derive(Deserialize)]
        struct GoogleGeminiCandidate {
            content: GoogleGeminiContent,
        }

        #[derive(Deserialize)]
        struct GoogleGeminiUsage {
            total_token_count: usize,
        }

        let request = GoogleGeminiRequest {
            contents: vec![GoogleGeminiContent {
                parts: vec![GoogleGeminiPart {
                    text: prompt.to_string(),
                }],
            }],
        };

        let base_url = model
            .base_url
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com");
        let response = self
            .client
            .post(format!(
                "{}/v1/models/{}/:generateContent?key={}",
                base_url, model.model_name, model.api_key
            ))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<GoogleGeminiResponse>()
            .await?;

        let candidate = response
            .candidates
            .first()
            .ok_or("No candidates in response")?;
        let part = candidate
            .content
            .parts
            .first()
            .ok_or("No parts in content")?;

        Ok(AIResponse {
            content: part.text.clone(),
            model: model.model_name.clone(),
            platform: AIPlatform::GoogleGemini,
            tokens_used: Some(response.usage_metadata.total_token_count),
        })
    }

    /// Generate response from Mistral
    async fn generate_mistral_response(
        &self,
        model: &AIModel,
        prompt: &str,
    ) -> Result<AIResponse, Box<dyn Error>> {
        // Mistral API request
        #[derive(Serialize)]
        struct MistralRequest {
            model: String,
            messages: Vec<MistralMessage>,
            temperature: f32,
        }

        #[derive(Serialize, Deserialize)]
        struct MistralMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct MistralResponse {
            choices: Vec<MistralChoice>,
            usage: MistralUsage,
        }

        #[derive(Deserialize)]
        struct MistralChoice {
            message: MistralMessage,
        }

        #[derive(Deserialize)]
        struct MistralUsage {
            total_tokens: usize,
        }

        let request = MistralRequest {
            model: model.model_name.clone(),
            messages: vec![MistralMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
        };

        let base_url = model
            .base_url
            .as_deref()
            .unwrap_or("https://api.mistral.ai");
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<MistralResponse>()
            .await?;

        let choice = response.choices.first().ok_or("No choices in response")?;

        Ok(AIResponse {
            content: choice.message.content.clone(),
            model: model.model_name.clone(),
            platform: AIPlatform::Mistral,
            tokens_used: Some(response.usage.total_tokens),
        })
    }

    /// Generate response from Ollama
    async fn generate_ollama_response(
        &self,
        model: &AIModel,
        prompt: &str,
    ) -> Result<AIResponse, Box<dyn Error>> {
        // Ollama API request
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
            model: String,
            done: bool,
        }

        let request = OllamaRequest {
            model: model.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let base_url = model
            .base_url
            .as_deref()
            .unwrap_or("http://localhost:11434");
        let response = self
            .client
            .post(format!("{}/api/generate", base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(AIResponse {
            content: response.response,
            model: response.model,
            platform: AIPlatform::Ollama,
            tokens_used: None,
        })
    }

    /// Add or update a model configuration
    pub fn add_model(&mut self, name: &str, model: AIModel) {
        self.models.insert(name.to_string(), model);
    }

    /// Add or update a prompt template
    pub fn add_template(&mut self, name: &str, template: PromptTemplate) {
        self.templates.insert(name.to_string(), template);
    }

    /// Set the default model to use
    pub fn set_default_model(&mut self, model_name: &str) {
        if self.models.contains_key(model_name) {
            self.default_model = model_name.to_string();
        }
    }
}
