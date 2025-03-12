// src/main.rs - Updated with proper error handling and CLI integration

mod error;
mod computer;
mod mock;
mod thread_computer;
mod api;
mod agent;
mod cli;

use tokio;
use computer::Computer;
use mock::MockComputer;
use thread_computer::ThreadComputer;
use std::io::{self, Write};
use std::env;
use std::collections::HashMap;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger if needed
    env_logger::init();
    
    // Load environment variables from .env file if it exists
    dotenv().ok();
    
    println!("OpenAI CUA Desktop Environment");
    
    // Run the CLI
    match cli::run().await {
        Ok(_) => {
            println!("CLI exited successfully");
        }
        Err(e) => {
            eprintln!("Error running CLI: {}", e);
            // Do not do test mode...
            // println!("Falling back to test mode...");
            // run_test_mode().await?;
        }
    }
    
    Ok(())
}

/// Run a simple test mode when the CLI fails to start
async fn run_test_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing desktop environment functionality");
    
    // Check what implementation to use
    let use_mock = env::var("USE_MOCK").unwrap_or_default() == "1";
    let use_thread = env::var("USE_THREAD").unwrap_or_default() == "1";
    
    if use_mock {
        run_with_mock().await?;
    } else if use_thread {
        match run_with_thread().await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error using thread-based implementation: {}", e);
                println!("Falling back to mock implementation...");
                run_with_mock().await?;
            }
        }
    } else {
        // Default to thread-based implementation as it's our new recommended approach
        println!("Using ThreadComputer by default. To use another implementation, set USE_MOCK=1");
        match run_with_thread().await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error using thread-based implementation: {}", e);
                println!("Falling back to mock implementation...");
                run_with_mock().await?;
            }
        }
    }
    
    Ok(())
}

async fn run_with_mock() -> Result<(), Box<dyn std::error::Error>> {
    println!("Using MockComputer for testing");
    
    // Create a mock computer
    let computer = MockComputer::new("linux", 1920, 1080);
    println!("Created MockComputer with dimensions: {:?}", computer.dimensions());
    
    // Test various operations
    println!("Moving cursor to (100, 100)...");
    computer.move_cursor(100, 100).await?;
    
    println!("Clicking at position (100, 100)...");
    computer.click(100, 100, "left").await?;
    
    println!("Typing text: Hello, world!");
    computer.type_text("Hello, world!").await?;
    
    println!("Waiting for 1 second...");
    computer.wait(1000).await?;
    
    println!("Taking a screenshot...");
    let screenshot = computer.screenshot().await?;
    println!("Screenshot taken (base64 length: {})", screenshot.len());
    
    println!("Final cursor position: {:?}", computer.cursor_position());
    
    // Add a little demo of drag operation
    println!("\nDemonstrating drag operation:");
    println!("  Starting at (200, 200)");
    println!("  Dragging to (300, 300)");
    
    let mut path = Vec::new();
    path.push(HashMap::from([("x".to_string(), 200), ("y".to_string(), 200)]));
    path.push(HashMap::from([("x".to_string(), 250), ("y".to_string(), 250)]));
    path.push(HashMap::from([("x".to_string(), 300), ("y".to_string(), 300)]));
    
    computer.drag(&path).await?;
    
    println!("Press Enter to exit...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(())
}

async fn run_with_thread() -> Result<(), Box<dyn std::error::Error>> {
    println!("Using thread-based Enigo implementation");
    
    // Create ThreadComputer
    let computer = ThreadComputer::new()?;
    println!("Created ThreadComputer with dimensions: {:?}", computer.dimensions());
    
    // Perform the same operations as with the other implementations
    println!("Moving cursor to (100, 100)...");
    computer.move_cursor(100, 100).await?;
    
    println!("Clicking at position (100, 100)...");
    computer.click(100, 100, "left").await?;
    
    println!("Typing text: Hello from thread-safe Enigo!");
    computer.type_text("Hello from thread-safe Enigo!").await?;
    
    println!("Waiting for 1 second...");
    computer.wait(1000).await?;
    
    println!("Taking a screenshot...");
    let screenshot = computer.screenshot().await?;
    println!("Screenshot taken (base64 length: {})", screenshot.len());
    
    // Add a little demo of drag operation
    println!("\nDemonstrating drag operation:");
    println!("  Starting at (200, 200)");
    println!("  Dragging to (300, 300)");
    
    let mut path = Vec::new();
    path.push(HashMap::from([("x".to_string(), 200), ("y".to_string(), 200)]));
    path.push(HashMap::from([("x".to_string(), 250), ("y".to_string(), 250)]));
    path.push(HashMap::from([("x".to_string(), 300), ("y".to_string(), 300)]));
    
    computer.drag(&path).await?;
    
    println!("Press Enter to exit...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(())
}