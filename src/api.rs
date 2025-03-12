// src/api.rs - Updated to match OpenAI CUA requirements

use crate::error::CuaError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use hyper::{body::to_bytes, Client, Request, Body, Method};
use hyper_tls::HttpsConnector;
use http::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

/// Response from the OpenAI API
#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub output: Vec<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Client for communicating with the OpenAI API
pub struct OpenAIClient {
    api_key: String,
    org_id: Option<String>,
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    model: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client with the specified API key and org ID
    pub fn new(api_key: String, org_id: Option<String>, model: Option<String>) -> Self {
        // Create HTTPS connector
        let https = HttpsConnector::new();
        
        // Create client
        let client = Client::builder().build(https);
        
        // Default model for CUA
        let model = model.unwrap_or_else(|| "computer-use-preview".to_string());
        
        Self {
            api_key,
            org_id,
            client,
            model,
        }
    }
    
    /// Create a new OpenAI client from environment variables
    pub fn from_env(model: Option<String>) -> Result<Self, CuaError> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| CuaError::Other("OPENAI_API_KEY environment variable not set".to_string()))?;
        
        let org_id = env::var("OPENAI_ORG").ok();
        
        Ok(Self::new(api_key, org_id, model))
    }
    
    /// Create a response using the Responses API
    pub async fn create_response(&self, input: &[Value], tools: &[Value]) -> Result<ApiResponse, CuaError> {
        let url = "https://api.openai.com/v1/responses";
        
        // Create the request body
        let body = json!({
            "model": self.model,
            "input": input,
            "tools": tools,
            "truncation": "auto"
        });
        
        // Create the request with individual headers
        let mut request_builder = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .header("Openai-Beta", "responses=v1");
        
        // Add org ID if provided
        if let Some(org_id) = &self.org_id {
            request_builder = request_builder.header("OpenAI-Organization", org_id);
        }
        
        // Build the request with the body
        let request = request_builder
            .body(Body::from(body.to_string()))
            .map_err(|e| CuaError::Other(format!("Failed to create request: {}", e)))?;
        
        // Send the request
        let response = self.client.request(request)
            .await
            .map_err(|e| CuaError::Other(format!("Failed to send request: {}", e)))?;
        
        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let body_bytes = to_bytes(response.into_body())
                .await
                .map_err(|e| CuaError::Other(format!("Failed to read error response: {}", e)))?;
            
            let error_text = String::from_utf8_lossy(&body_bytes);
            
            return Err(CuaError::Other(format!(
                "API returned error {}: {}",
                status,
                error_text
            )));
        }
        
        // Parse the response
        let body_bytes = to_bytes(response.into_body())
            .await
            .map_err(|e| CuaError::Other(format!("Failed to read response: {}", e)))?;
        
        let api_response = serde_json::from_slice::<ApiResponse>(&body_bytes)
            .map_err(|e| CuaError::Other(format!("Failed to parse response: {}", e)))?;
        
        Ok(api_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new(
            "test_key".to_string(),
            Some("test_org".to_string()),
            Some("test_model".to_string())
        );
        
        assert_eq!(client.api_key, "test_key");
        assert_eq!(client.org_id, Some("test_org".to_string()));
        assert_eq!(client.model, "test_model");
    }
}