//! Mock implementation of the Computer trait for testing purposes

use crate::computer::Computer;
use crate::error::CuaError;
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::time::sleep;
use std::time::Duration;
use async_trait::async_trait;

/// A mock implementation of the Computer trait for testing
pub struct MockComputer {
    environment: String,
    dimensions: (u32, u32),
    cursor_position: RwLock<(i32, i32)>,
    current_url: RwLock<String>,
}

impl MockComputer {
    /// Create a new MockComputer with the specified environment and dimensions
    pub fn new(environment: &str, width: u32, height: u32) -> Self {
        let current_url = if environment == "browser" {
            "https://example.com".to_string()
        } else {
            "".to_string()
        };
        
        Self {
            environment: environment.to_string(),
            dimensions: (width, height),
            cursor_position: RwLock::new((0, 0)),
            current_url: RwLock::new(current_url),
        }
    }
    
    /// Get the current cursor position
    pub fn cursor_position(&self) -> (i32, i32) {
        *self.cursor_position.read().unwrap()
    }
    
    /// Set current URL (for browser environments)
    pub fn set_url(&self, url: &str) {
        if self.environment == "browser" {
            *self.current_url.write().unwrap() = url.to_string();
        }
    }
}

#[async_trait]
impl Computer for MockComputer {
    fn environment(&self) -> &str {
        &self.environment
    }
    
    fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
    
    async fn screenshot(&self) -> Result<String, CuaError> {
        // Return a mock base64-encoded string
        println!("MockComputer: Taking screenshot");
        Ok("bW9ja3NjcmVlbnNob3Q=".to_string()) // "mockscreenshot" in base64
    }
    
    async fn click(&self, x: i32, y: i32, button: &str) -> Result<(), CuaError> {
        println!("MockComputer: Clicking at ({}, {}) with button: {}", x, y, button);
        // Update cursor position
        *self.cursor_position.write().unwrap() = (x, y);
        Ok(())
    }
    
    async fn double_click(&self, x: i32, y: i32) -> Result<(), CuaError> {
        println!("MockComputer: Double-clicking at ({}, {})", x, y);
        // Update cursor position
        *self.cursor_position.write().unwrap() = (x, y);
        Ok(())
    }
    
    async fn scroll(&self, x: i32, y: i32, scroll_x: i32, scroll_y: i32) -> Result<(), CuaError> {
        println!("MockComputer: Scrolling at ({}, {}) with delta ({}, {})", 
                x, y, scroll_x, scroll_y);
        // Update cursor position
        *self.cursor_position.write().unwrap() = (x, y);
        Ok(())
    }
    
    async fn type_text(&self, text: &str) -> Result<(), CuaError> {
        println!("MockComputer: Typing text: {}", text);
        Ok(())
    }
    
    async fn wait(&self, ms: u32) -> Result<(), CuaError> {
        println!("MockComputer: Waiting for {} ms", ms);
        sleep(Duration::from_millis(ms as u64)).await;
        Ok(())
    }
    
    async fn move_cursor(&self, x: i32, y: i32) -> Result<(), CuaError> {
        println!("MockComputer: Moving cursor to ({}, {})", x, y);
        // Update cursor position
        *self.cursor_position.write().unwrap() = (x, y);
        Ok(())
    }
    
    async fn keypress(&self, keys: &[String]) -> Result<(), CuaError> {
        println!("MockComputer: Pressing keys: {:?}", keys);
        Ok(())
    }
    
    async fn drag(&self, path: &[HashMap<String, i32>]) -> Result<(), CuaError> {
        println!("MockComputer: Dragging along path with {} points", path.len());
        for (i, point) in path.iter().enumerate() {
            let x = *point.get("x").unwrap_or(&0);
            let y = *point.get("y").unwrap_or(&0);
            println!("  Point {}: ({}, {})", i, x, y);
            
            // Update cursor position for the last point
            if i == path.len() - 1 {
                *self.cursor_position.write().unwrap() = (x, y);
            }
        }
        Ok(())
    }
    
    async fn get_current_url(&self) -> Result<String, CuaError> {
        // Return the current URL or an empty string for non-browser environments
        let url = self.current_url.read().unwrap().clone();
        println!("MockComputer: Getting current URL: {}", url);
        Ok(url)
    }
    
    // Add a new method to handle browser navigation
    // This is not in the Computer trait, but we'll add it to enable graceful handling
    async fn goto(&self, url: &str) -> Result<(), CuaError> {
        if self.environment == "browser" {
            println!("MockComputer: Navigating to URL: {}", url);
            *self.current_url.write().unwrap() = url.to_string();
            Ok(())
        } else {
            println!("MockComputer: Cannot navigate to URL in non-browser environment");
            Err(CuaError::ActionError("Cannot navigate to URL in non-browser environment".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_computer_creation() {
        let computer = MockComputer::new("windows", 1920, 1080);
        assert_eq!(computer.environment(), "windows");
        assert_eq!(computer.dimensions(), (1920, 1080));
    }
    
    #[tokio::test]
    async fn test_mock_cursor_movement() {
        let computer = MockComputer::new("linux", 1024, 768);
        
        // Initial position should be (0, 0)
        assert_eq!(computer.cursor_position(), (0, 0));
        
        // Move cursor and check position
        computer.move_cursor(100, 200).await.unwrap();
        assert_eq!(computer.cursor_position(), (100, 200));
    }
    
    #[tokio::test]
    async fn test_mock_browser_url() {
        let computer = MockComputer::new("browser", 1024, 768);
        
        // Check default URL
        assert_eq!(computer.get_current_url().await.unwrap(), "https://example.com");
        
        // Navigate to a new URL
        computer.goto("https://google.com").await.unwrap();
        assert_eq!(computer.get_current_url().await.unwrap(), "https://google.com");
    }
}