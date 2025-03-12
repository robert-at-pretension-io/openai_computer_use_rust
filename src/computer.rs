//! Computer trait defining the interface for desktop control

use crate::error::CuaError;
use std::collections::HashMap;
use async_trait::async_trait;

/// Trait defining the interface for controlling a computer
#[async_trait]
pub trait Computer: Send + Sync {
    /// Get the environment type (windows, mac, linux, browser)
    fn environment(&self) -> &str;
    
    /// Get the dimensions of the screen
    fn dimensions(&self) -> (u32, u32);
    
    /// Take a screenshot and return it as a base64-encoded string
    async fn screenshot(&self) -> Result<String, CuaError>;
    
    /// Click at the specified coordinates
    async fn click(&self, x: i32, y: i32, button: &str) -> Result<(), CuaError>;
    
    /// Double-click at the specified coordinates
    async fn double_click(&self, x: i32, y: i32) -> Result<(), CuaError>;
    
    /// Scroll at the specified coordinates
    async fn scroll(&self, x: i32, y: i32, scroll_x: i32, scroll_y: i32) -> Result<(), CuaError>;
    
    /// Type the specified text
    async fn type_text(&self, text: &str) -> Result<(), CuaError>;
    
    /// Wait for the specified duration in milliseconds
    async fn wait(&self, ms: u32) -> Result<(), CuaError>;
    
    /// Move the cursor to the specified coordinates
    async fn move_cursor(&self, x: i32, y: i32) -> Result<(), CuaError>;
    
    /// Press the specified keys
    async fn keypress(&self, keys: &[String]) -> Result<(), CuaError>;
    
    /// Drag from one point to another
    async fn drag(&self, path: &[HashMap<String, i32>]) -> Result<(), CuaError>;
    
    /// Get the current URL (for browser environments)
    async fn get_current_url(&self) -> Result<String, CuaError>;
    
    /// Navigate to a URL (for browser environments)
    /// Default implementation returns an error for non-browser environments
    async fn goto(&self, _url: &str) -> Result<(), CuaError> {
        if self.environment() == "browser" {
            Err(CuaError::ActionError("goto not implemented for this browser environment".to_string()))
        } else {
            Err(CuaError::ActionError("Cannot navigate to URL in non-browser environment".to_string()))
        }
    }
}