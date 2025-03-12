//! Thread-based implementation of the Computer trait using Enigo

use crate::computer::Computer;
use crate::error::CuaError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;
use tokio::time::sleep;
use std::time::Duration;
use enigo::{Enigo, MouseControllable, KeyboardControllable, MouseButton, Key};
use screenshots::Screen;
use base64::{engine::general_purpose, Engine};
use async_trait::async_trait;
use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use std::io::Cursor;

/// Commands that can be sent to the input thread
enum InputCommand {
    Click {
        x: i32, 
        y: i32, 
        button: String,
        response: oneshot::Sender<Result<(), CuaError>>,
    },
    DoubleClick {
        x: i32,
        y: i32,
        response: oneshot::Sender<Result<(), CuaError>>,
    },
    Scroll {
        x: i32,
        y: i32,
        scroll_x: i32,
        scroll_y: i32,
        response: oneshot::Sender<Result<(), CuaError>>,
    },
    TypeText {
        text: String,
        response: oneshot::Sender<Result<(), CuaError>>,
    },
    MoveCursor {
        x: i32,
        y: i32,
        response: oneshot::Sender<Result<(), CuaError>>,
    },
    Keypress {
        keys: Vec<String>,
        response: oneshot::Sender<Result<(), CuaError>>,
    },
    Drag {
        path: Vec<HashMap<String, i32>>,
        response: oneshot::Sender<Result<(), CuaError>>,
    },
    Screenshot {
        response: oneshot::Sender<Result<String, CuaError>>,
    },
    Shutdown,
}

/// Map keys from CUA format to Enigo format
fn map_key(key: &str) -> Option<Key> {
    match key.to_lowercase().as_str() {
        "alt" => Some(Key::Alt),
        "backspace" => Some(Key::Backspace),
        "capslock" => Some(Key::CapsLock),
        "ctrl" => Some(Key::Control),
        "delete" => Some(Key::Delete),
        "end" => Some(Key::End),
        "enter" => Some(Key::Return),
        "esc" => Some(Key::Escape),
        "home" => Some(Key::Home),
        "option" => Some(Key::Alt),
        "shift" => Some(Key::Shift),
        "space" => Some(Key::Space),
        "super" | "win" | "cmd" => Some(Key::Meta),
        "tab" => Some(Key::Tab),
        "arrowdown" => Some(Key::DownArrow),
        "arrowleft" => Some(Key::LeftArrow),
        "arrowright" => Some(Key::RightArrow),
        "arrowup" => Some(Key::UpArrow),
        _ => None,
    }
}

/// Thread-safe computer implementation
pub struct ThreadComputer {
    /// Channel to send commands to the input thread
    command_sender: Sender<InputCommand>,
    /// Environment type
    environment: String,
    /// Screen dimensions
    dimensions: (u32, u32),
    /// Screen ID
    screen_id: usize,
    /// Cursor position
    cursor_position: Arc<Mutex<(i32, i32)>>,
}

impl ThreadComputer {
    /// Create a new ThreadComputer
    pub fn new() -> Result<Self, CuaError> {
        // Get screen information
        let screens = Screen::all().map_err(|e| 
            CuaError::Other(format!("Failed to get screen information: {}", e))
        )?;
        
        if screens.is_empty() {
            return Err(CuaError::Other("No screens detected".to_string()));
        }
        
        // Use the primary screen
        let screen = &screens[0];
        let dimensions = (screen.display_info.width, screen.display_info.height);
        let screen_id = 0;
        
        // Create a channel for sending commands to the input thread
        let (tx, mut rx) = mpsc::channel::<InputCommand>(100);
        
        // Shared cursor position
        let cursor_position = Arc::new(Mutex::new((0, 0)));
        let cursor_position_clone = cursor_position.clone();
        
        // Spawn the input thread
        thread::spawn(move || {
            // Create Enigo instance for mouse/keyboard control
            let mut enigo = Enigo::new();
            
            println!("Input thread started");
            
            // Process commands from the channel
            while let Some(cmd) = rx.blocking_recv() {
                match cmd {
                    InputCommand::Click { x, y, button, response } => {
                        println!("DEBUG: Processing InputCommand::Click at ({}, {}) with button: {}", x, y, button);
                        let result = (|| {
                            // Move to position first
                            enigo.mouse_move_to(x, y);
                            
                            // Update cursor position
                            *cursor_position_clone.lock().unwrap() = (x, y);
                            
                            // Determine which button to click
                            let mouse_button = match button.to_lowercase().as_str() {
                                "right" => MouseButton::Right,
                                "middle" => MouseButton::Middle,
                                _ => MouseButton::Left, // Default to left click for any other value
                            };
                            
                            // Click the button
                            enigo.mouse_click(mouse_button);
                            
                            Ok(())
                        })();
                        
                        let _ = response.send(result);
                    }
                    
                    InputCommand::DoubleClick { x, y, response } => {
                        let result = (|| {
                            // Move to position first
                            enigo.mouse_move_to(x, y);
                            
                            // Update cursor position
                            *cursor_position_clone.lock().unwrap() = (x, y);
                            
                            // Double click (two quick clicks)
                            enigo.mouse_click(MouseButton::Left);
                            thread::sleep(Duration::from_millis(10)); // Short delay between clicks
                            enigo.mouse_click(MouseButton::Left);
                            
                            Ok(())
                        })();
                        
                        let _ = response.send(result);
                    }
                    
                    InputCommand::Scroll { x, y, scroll_x, scroll_y, response } => {
                        let result = (|| {
                            // Move to position first
                            enigo.mouse_move_to(x, y);
                            
                            // Update cursor position
                            *cursor_position_clone.lock().unwrap() = (x, y);
                            
                            // Scroll
                            // Note: Enigo's scroll direction is opposite to what most users expect
                            // So we negate the scroll values
                            if scroll_x != 0 {
                                enigo.mouse_scroll_x(-(scroll_x / 3));
                            }
                            
                            if scroll_y != 0 {
                                enigo.mouse_scroll_y(scroll_y / 3);
                            }
                            
                            Ok(())
                        })();
                        
                        let _ = response.send(result);
                    }
                    
                    InputCommand::TypeText { text, response } => {
                        println!("DEBUG: Processing InputCommand::TypeText with text: {}", text);
                        let result = (|| {
                            enigo.key_sequence(&text);
                            Ok(())
                        })();
                        
                        let _ = response.send(result);
                    }
                    
                    InputCommand::MoveCursor { x, y, response } => {
                        let result = (|| {
                            enigo.mouse_move_to(x, y);
                            
                            // Update cursor position
                            *cursor_position_clone.lock().unwrap() = (x, y);
                            
                            Ok(())
                        })();
                        
                        let _ = response.send(result);
                    }
                    
                    InputCommand::Keypress { keys, response } => {
                        let result = (|| {
                            for key in keys {
                                // Try to map to a special key
                                if let Some(special_key) = map_key(&key) {
                                    enigo.key_down(special_key);
                                    enigo.key_up(special_key);
                                } else if key.len() == 1 {
                                    // Single character, just type it
                                    enigo.key_sequence(&key);
                                } else {
                                    // Unknown key
                                    return Err(CuaError::ActionError(format!(
                                        "Unknown key: {}", key
                                    )));
                                }
                            }
                            
                            Ok(())
                        })();
                        
                        let _ = response.send(result);
                    }
                    
                    InputCommand::Drag { path, response } => {
                        let result = (|| {
                            if path.is_empty() {
                                return Ok(());
                            }
                            
                            // Get the first point
                            let first_point = path.first().unwrap();
                            let start_x = *first_point.get("x").unwrap_or(&0);
                            let start_y = *first_point.get("y").unwrap_or(&0);
                            
                            // Move to the starting point
                            enigo.mouse_move_to(start_x, start_y);
                            
                            // Press and hold the mouse button
                            enigo.mouse_down(MouseButton::Left);
                            
                            // Move to each subsequent point
                            for point in path.iter().skip(1) {
                                let x = *point.get("x").unwrap_or(&0);
                                let y = *point.get("y").unwrap_or(&0);
                                
                                enigo.mouse_move_to(x, y);
                                
                                // Small delay to make the drag smoother
                                thread::sleep(Duration::from_millis(5));
                            }
                            
                            // Release the mouse button
                            enigo.mouse_up(MouseButton::Left);
                            
                            // Update cursor position with the last point
                            if let Some(last_point) = path.last() {
                                let x = *last_point.get("x").unwrap_or(&0);
                                let y = *last_point.get("y").unwrap_or(&0);
                                *cursor_position_clone.lock().unwrap() = (x, y);
                            }
                            
                            Ok(())
                        })();
                        
                        let _ = response.send(result);
                    }
                    
                    InputCommand::Screenshot { response } => {
                        let result = (|| {
                            // Get all screens
                            let screens = Screen::all().map_err(|e| 
                                CuaError::ScreenshotError(format!("Failed to get screen information: {}", e))
                            )?;
                            
                            // Capture the screen with the specified ID
                            if screen_id >= screens.len() {
                                return Err(CuaError::ScreenshotError(format!(
                                    "Invalid screen ID: {}, only {} screens available",
                                    screen_id, screens.len()
                                )));
                            }
                            
                            let screen = &screens[screen_id];
                            let image = screen.capture().map_err(|e| 
                                CuaError::ScreenshotError(format!("Failed to capture screenshot: {}", e))
                            )?;
                            
                            // Convert image to PNG using the image crate
                            let mut buffer = Vec::new();
                            let cursor = Cursor::new(&mut buffer);
                            
                            let width = image.width();
                            let height = image.height();
                            let data = image.rgba();
                            
                            // Create a PNG encoder and encode the image
                            let encoder = PngEncoder::new(cursor);
                            encoder.write_image(
                                data,
                                width,
                                height,
                                image::ColorType::Rgba8,
                            ).map_err(|e| 
                                CuaError::ScreenshotError(format!("Failed to encode PNG: {}", e))
                            )?;
                            
                            // Base64 encode the PNG data
                            Ok(general_purpose::STANDARD.encode(&buffer))
                        })();
                        
                        let _ = response.send(result);
                    }
                    
                    InputCommand::Shutdown => {
                        println!("Input thread shutting down");
                        break;
                    }
                }
            }
            
            println!("Input thread terminated");
        });
        
        Ok(Self {
            command_sender: tx,
            environment: "linux".to_string(),
            dimensions,
            screen_id,
            cursor_position,
        })
    }
    
    /// Get the current cursor position
    pub fn cursor_position(&self) -> (i32, i32) {
        *self.cursor_position.lock().unwrap()
    }
}

impl Drop for ThreadComputer {
    fn drop(&mut self) {
        // Send shutdown command to the input thread
        let _ = self.command_sender.try_send(InputCommand::Shutdown);
    }
}

#[async_trait]
impl Computer for ThreadComputer {
    fn environment(&self) -> &str {
        &self.environment
    }
    
    fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
    
    async fn screenshot(&self) -> Result<String, CuaError> {
        let (tx, rx) = oneshot::channel();
        
        self.command_sender.send(InputCommand::Screenshot {
            response: tx,
        }).await.map_err(|_| CuaError::ActionError("Failed to send screenshot command: desktop input thread is shutting down".to_string()))?;
        
        rx.await.map_err(|_| CuaError::ActionError("Failed to receive screenshot response: desktop input thread is not available".to_string()))?
    }
    
    async fn click(&self, x: i32, y: i32, button: &str) -> Result<(), CuaError> {
        let (tx, rx) = oneshot::channel();
        
        self.command_sender.send(InputCommand::Click {
            x,
            y,
            button: button.to_string(),
            response: tx,
        }).await.map_err(|_| CuaError::ActionError("Failed to send click command: desktop input thread is shutting down".to_string()))?;
        
        rx.await.map_err(|_| CuaError::ActionError("Failed to receive click response: desktop input thread is not available".to_string()))?
    }
    
    async fn double_click(&self, x: i32, y: i32) -> Result<(), CuaError> {
        let (tx, rx) = oneshot::channel();
        
        self.command_sender.send(InputCommand::DoubleClick {
            x,
            y,
            response: tx,
        }).await.map_err(|_| CuaError::ActionError("Failed to send double click command: desktop input thread is shutting down".to_string()))?;
        
        rx.await.map_err(|_| CuaError::ActionError("Failed to receive double click response: desktop input thread is not available".to_string()))?
    }
    
    async fn scroll(&self, x: i32, y: i32, scroll_x: i32, scroll_y: i32) -> Result<(), CuaError> {
        let (tx, rx) = oneshot::channel();
        
        self.command_sender.send(InputCommand::Scroll {
            x,
            y,
            scroll_x,
            scroll_y,
            response: tx,
        }).await.map_err(|_| CuaError::ActionError("Failed to send scroll command: desktop input thread is shutting down".to_string()))?;
        
        rx.await.map_err(|_| CuaError::ActionError("Failed to receive scroll response: desktop input thread is not available".to_string()))?
    }
    
    async fn type_text(&self, text: &str) -> Result<(), CuaError> {
        let (tx, rx) = oneshot::channel();
        
        self.command_sender.send(InputCommand::TypeText {
            text: text.to_string(),
            response: tx,
        }).await.map_err(|_| CuaError::ActionError("Failed to send type text command: desktop input thread is shutting down".to_string()))?;
        
        rx.await.map_err(|_| CuaError::ActionError("Failed to receive type text response: desktop input thread is not available".to_string()))?
    }
    
    async fn wait(&self, ms: u32) -> Result<(), CuaError> {
        sleep(Duration::from_millis(ms as u64)).await;
        Ok(())
    }
    
    async fn move_cursor(&self, x: i32, y: i32) -> Result<(), CuaError> {
        let (tx, rx) = oneshot::channel();
        
        self.command_sender.send(InputCommand::MoveCursor {
            x,
            y,
            response: tx,
        }).await.map_err(|_| CuaError::ActionError("Failed to send move cursor command: desktop input thread is shutting down".to_string()))?;
        
        rx.await.map_err(|_| CuaError::ActionError("Failed to receive move cursor response: desktop input thread is not available".to_string()))?
    }
    
    async fn keypress(&self, keys: &[String]) -> Result<(), CuaError> {
        let (tx, rx) = oneshot::channel();
        
        self.command_sender.send(InputCommand::Keypress {
            keys: keys.to_vec(),
            response: tx,
        }).await.map_err(|_| CuaError::ActionError("Failed to send keypress command: desktop input thread is shutting down".to_string()))?;
        
        rx.await.map_err(|_| CuaError::ActionError("Failed to receive keypress response: desktop input thread is not available".to_string()))?
    }
    
    async fn drag(&self, path: &[HashMap<String, i32>]) -> Result<(), CuaError> {
        let (tx, rx) = oneshot::channel();
        
        self.command_sender.send(InputCommand::Drag {
            path: path.to_vec(),
            response: tx,
        }).await.map_err(|_| CuaError::ActionError("Failed to send drag command: desktop input thread is shutting down".to_string()))?;
        
        rx.await.map_err(|_| CuaError::ActionError("Failed to receive drag response: desktop input thread is not available".to_string()))?
    }
    
    async fn get_current_url(&self) -> Result<String, CuaError> {
        // Not applicable for desktop environments
        Ok("".to_string())
    }
}
