// src/agent.rs - Updated to match OpenAI CUA approach

use crate::computer::Computer;
use crate::api::OpenAIClient;
use crate::error::CuaError;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;

/// Safety check callback type
pub type SafetyCheckCallback = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// Default safety check callback that always returns true
pub fn default_safety_check_callback(_message: &str) -> bool {
    println!("Safety check: {}", _message);
    println!("Do you want to allow this action? (y/n): ");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or_default();
    
    input.trim().to_lowercase() == "y"
}

/// Agent that manages the interaction between the model and computer
pub struct Agent {
    client: OpenAIClient,
    computer: Box<dyn Computer>,
    tools: Vec<Value>,
    print_steps: bool,
    debug: bool,
    show_images: bool,
    acknowledge_safety_check: SafetyCheckCallback,
}

impl Agent {
    /// Create a new agent with the specified client, computer, and tools
    pub fn new(
        client: OpenAIClient,
        computer: Box<dyn Computer>,
        mut tools: Vec<Value>,
        acknowledge_safety_check: Option<SafetyCheckCallback>,
    ) -> Self {
        // Add computer tool
        tools.push(json!({
            "type": "computer-preview",
            "display_width": computer.dimensions().0,
            "display_height": computer.dimensions().1,
            "environment": computer.environment(),
        }));
        
        Self {
            client,
            computer,
            tools,
            print_steps: true,
            debug: false,
            show_images: false,
            acknowledge_safety_check: acknowledge_safety_check
                .unwrap_or_else(|| Box::new(default_safety_check_callback)),
        }
    }
    
    /// Set whether to print steps
    pub fn with_print_steps(mut self, print_steps: bool) -> Self {
        self.print_steps = print_steps;
        self
    }
    
    /// Set whether to print debug information
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    
    /// Set whether to show images
    pub fn with_show_images(mut self, show_images: bool) -> Self {
        self.show_images = show_images;
        self
    }
    
    /// Debug print a value
    fn debug_print(&self, value: &impl fmt::Debug) {
        if self.debug {
            println!("{:?}", value);
        }
    }
    
    /// Handle an item from the API response
    async fn handle_item(&self, item: &Value) -> Result<Vec<Value>, CuaError> {
        let mut new_items = Vec::new();
        
        // Check the item type
        let item_type = item.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        
        match item_type {
            "message" => {
                if self.debug {
                    println!("DEBUG: Handling 'message' item: {:?}", item);
                }
                if self.print_steps {
                    if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
                        if let Some(text_obj) = content.first() {
                            if let Some(text) = text_obj.get("text").and_then(|t| t.as_str()) {
                                println!("{}", text);
                            }
                        }
                    }
                }
            }
            "function_call" => {
                if let (Some(name), Some(arguments), Some(call_id)) = (
                    item.get("name").and_then(|n| n.as_str()),
                    item.get("arguments").and_then(|a| a.as_str()),
                    item.get("call_id"),
                ) {
                    if self.debug {
                        println!("DEBUG: Handling 'function_call' item with name: {}", name);
                    }
                    if self.print_steps {
                        println!("Function call: {}({})", name, arguments);
                    }
                    
                    // For now, we return a hardcoded success response
                    // In a real implementation, we'd handle specific function calls here
                    new_items.push(json!({
                        "type": "function_call_output",
                        "call_id": call_id,
                        "output": "success",
                    }));
                }
            }
            "computer_call" => {
                if let (Some(action), Some(call_id)) = (
                    item.get("action"),
                    item.get("call_id"),
                ) {
                    if let Some(action_type) = action.get("type").and_then(|t| t.as_str()) {
                        if self.print_steps {
                            println!("Computer action: {}", action_type);
                        }
                        
                        // Handle safety checks
                        let mut acknowledged_safety_checks = Vec::new();
                        if let Some(pending_checks) = item.get("pending_safety_checks").and_then(|c| c.as_array()) {
                            for check in pending_checks {
                                if let Some(message) = check.get("message").and_then(|m| m.as_str()) {
                                    if self.print_steps {
                                        println!("Safety check: {}", message);
                                    }
                                    
                                    if !(self.acknowledge_safety_check)(message) {
                                        return Err(CuaError::Other(format!(
                                            "Safety check failed: {}", message
                                        )));
                                    }
                                    
                                    acknowledged_safety_checks.push(check.clone());
                                }
                            }
                        }
                        
                        // Perform the action based on the type
                        match action_type {
                            // Handle explicit screenshot request
                            "screenshot" => {
                                // No action needed here, we'll take the screenshot below
                                if self.print_steps {
                                    println!("Taking screenshot as requested by the model");
                                }
                            },
                            "click" => {
                                let x = action.get("x").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
                                let y = action.get("y").and_then(|y| y.as_i64()).unwrap_or(0) as i32;
                                let button = action.get("button").and_then(|b| b.as_str()).unwrap_or("left");
                                if self.debug {
                                    println!("DEBUG: Processing click command at ({}, {}) with button: {}", x, y, button);
                                }
                                self.computer.click(x, y, button).await?;
                            }
                            "double_click" => {
                                let x = action.get("x").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
                                let y = action.get("y").and_then(|y| y.as_i64()).unwrap_or(0) as i32;
                                
                                self.computer.double_click(x, y).await?;
                            }
                            "scroll" => {
                                let x = action.get("x").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
                                let y = action.get("y").and_then(|y| y.as_i64()).unwrap_or(0) as i32;
                                let scroll_x = action.get("scroll_x").and_then(|sx| sx.as_i64()).unwrap_or(0) as i32;
                                let scroll_y = action.get("scroll_y").and_then(|sy| sy.as_i64()).unwrap_or(0) as i32;
                                
                                self.computer.scroll(x, y, scroll_x, scroll_y).await?;
                            }
                            "type" => {
                                let text = action.get("text").and_then(|t| t.as_str()).unwrap_or("");
                                
                                self.computer.type_text(text).await?;
                            }
                            "wait" => {
                                let ms = action.get("ms").and_then(|m| m.as_u64()).unwrap_or(1000) as u32;
                                
                                self.computer.wait(ms).await?;
                            }
                            "move" => {
                                let x = action.get("x").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
                                let y = action.get("y").and_then(|y| y.as_i64()).unwrap_or(0) as i32;
                                
                                self.computer.move_cursor(x, y).await?;
                            }
                            "keypress" => {
                                if let Some(keys) = action.get("keys").and_then(|k| k.as_array()) {
                                    let key_strings: Vec<String> = keys
                                        .iter()
                                        .filter_map(|k| k.as_str().map(|s| s.to_string()))
                                        .collect();
                                    
                                    self.computer.keypress(&key_strings).await?;
                                }
                            }
                            "drag" => {
                                if let Some(path) = action.get("path").and_then(|p| p.as_array()) {
                                    if self.debug {
                                        println!("DEBUG: Processing drag command with {} points", path.len());
                                    }
                                    let path_points: Vec<HashMap<String, i32>> = path
                                        .iter()
                                        .filter_map(|point| {
                                            if let Some(obj) = point.as_object() {
                                                let mut point_map = HashMap::new();
                                                if let (Some(x), Some(y)) = (
                                                    obj.get("x").and_then(|x| x.as_i64()),
                                                    obj.get("y").and_then(|y| y.as_i64()),
                                                ) {
                                                    point_map.insert("x".to_string(), x as i32);
                                                    point_map.insert("y".to_string(), y as i32);
                                                    Some(point_map)
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    
                                    self.computer.drag(&path_points).await?;
                                }
                            }
                            "goto" => {
                                if let Some(url) = action.get("url").and_then(|u| u.as_str()) {
                                    if self.print_steps {
                                        println!("Attempted to navigate to URL: {}", url);
                                        println!("Note: Direct URL navigation is not implemented. Please add browser navigation capabilities to your Computer implementation.");
                                    }
                                    // This is where you would add browser navigation functionality
                                    // Since we don't have a browser implementation yet, we'll just acknowledge
                                    // Instead of returning an error, we'll just continue
                                }
                            },
                            _ => {
                                if self.print_steps {
                                    println!("Unknown action type '{}' requested by the model", action_type);
                                    println!("This action is not implemented, but we'll continue with a screenshot");
                                }
                                // Instead of returning an error, we'll just continue
                                // This makes the agent more resilient to unknown action types
                            }
                        }
                        
                        // Take a screenshot
                        let screenshot_base64 = self.computer.screenshot().await?;
                        
                        // Create the response
                        let mut call_output = json!({
                            "type": "computer_call_output",
                            "call_id": call_id,
                            "acknowledged_safety_checks": acknowledged_safety_checks,
                            "output": {
                                "type": "input_image",
                                "image_url": format!("data:image/png;base64,{}", screenshot_base64),
                            },
                        });
                        
                        // Add current URL for browser environments
                        if self.computer.environment() == "browser" {
                            let current_url = self.computer.get_current_url().await?;
                            if let Some(output) = call_output.get_mut("output").and_then(|o| o.as_object_mut()) {
                                output.insert("current_url".to_string(), json!(current_url));
                            }
                        }
                        
                        new_items.push(call_output);
                    }
                }
            }
            _ => {}
        }
        
        Ok(new_items)
    }
    
    /// Run the agent for a single turn
    pub async fn run_full_turn(&self, input_items: &[Value]) -> Result<Vec<Value>, CuaError> {
        // Create a copy of input items
        let mut all_items = input_items.to_vec();
        
        // Use a set of processed IDs to avoid duplicates
        let mut processed_ids = std::collections::HashSet::new();
        
        // Save original input IDs
        for item in &all_items {
            if let Some(id) = item.get("id").and_then(|id| id.as_str()) {
                processed_ids.insert(id.to_string());
            }
        }
        
        // Keep looping until we get a final assistant response
        loop {
            // Debug print current state
            if self.debug {
                self.debug_print(&all_items);
            }
            
            // Create a request to the API
            let response = self.client.create_response(&all_items, &self.tools).await?;
            
            if self.debug {
                self.debug_print(&response);
            }
            
            let mut new_items = Vec::new();
            
            // Add the output to new items, checking for duplicates
            for item in response.output {
                // Skip items we've already processed
                if let Some(id) = item.get("id").and_then(|id| id.as_str()) {
                    if processed_ids.contains(id) {
                        continue;
                    }
                    processed_ids.insert(id.to_string());
                }
                
                new_items.push(item.clone());
                
                // Handle each item
                let handled_items = self.handle_item(&item).await?;
                
                // Also check for duplicates in handled items
                for handled_item in handled_items {
                    if let Some(id) = handled_item.get("id").and_then(|id| id.as_str()) {
                        if processed_ids.contains(id) {
                            continue;
                        }
                        processed_ids.insert(id.to_string());
                    }
                    new_items.push(handled_item);
                }
            }
            
            // Check if we got a final response
            if let Some(last_item) = new_items.last() {
                if last_item.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                    // Append new items to all_items for the result
                    all_items.extend(new_items);
                    break;
                }
            }
            
            // Update all items for the next iteration
            all_items.extend(new_items);
        }
        
        Ok(all_items)
    }
    
    /// Run the agent with the specified input
    pub async fn run(&self, input: &str) -> Result<Vec<Value>, CuaError> {
        // Create initial items
        let items = vec![json!({
            "role": "user",
            "content": input,
        })];
        
        // Run a turn with the input
        self.run_full_turn(&items).await
    }
    
    /// Run the agent interactively
    pub async fn run_interactive(&self) -> Result<(), CuaError> {
        let mut items = Vec::new();
        
        println!("OpenAI CUA Agent");
        println!("Type 'exit' to quit");
        
        loop {
            // Get input from user
            print!("> ");
            std::io::Write::flush(&mut std::io::stdout())?;
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            let input = input.trim();
            if input.eq_ignore_ascii_case("exit") {
                break;
            }
            
            // Add input to items
            items.push(json!({
                "role": "user",
                "content": input,
            }));
            
            // Run a turn with the input
            items = self.run_full_turn(&items).await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockComputer;
    
    // To run these tests, you need to have an OpenAI API key
    // and the API must support the CUA model
    #[tokio::test]
    #[ignore] // Ignore by default since it requires API credentials
    async fn test_agent_run() {
        // This is a simple test of the agent
        // It requires a valid API key and org ID
        
        // Set up API key and org ID
        std::env::set_var("OPENAI_API_KEY", "YOUR_API_KEY");
        std::env::set_var("OPENAI_ORG", "YOUR_ORG_ID");
        
        // Create client
        let client = OpenAIClient::from_env(None).unwrap();
        
        // Create computer
        let computer = MockComputer::new("linux", 1920, 1080);
        
        // Create agent
        let agent = Agent::new(
            client,
            Box::new(computer),
            Vec::new(),
            None,
        );
        
        // Run the agent
        let result = agent.run("Hello, world!").await;
        
        // Check if the agent ran successfully
        assert!(result.is_ok());
    }
}
