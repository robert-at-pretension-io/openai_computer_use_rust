// src/cli.rs - Updated to match OpenAI CUA approach

use crate::api::OpenAIClient;
use crate::agent::{Agent, SafetyCheckCallback};
use crate::computer::Computer;
use crate::mock::MockComputer;
use crate::error::CuaError;
use crate::thread_computer::ThreadComputer;
use std::io::{self, Write};
use std::env;
use dotenv::dotenv;

/// Run the CLI
pub async fn run() -> Result<(), CuaError> {
    // Load environment variables from .env file if it exists
    dotenv().ok();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut use_mock = false;
    let mut debug = false;
    let mut show_images = false;
    let mut input: Option<String> = None;
    let mut model: Option<String> = None;
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--mock" => {
                use_mock = true;
            }
            "--debug" => {
                debug = true;
            }
            "--show" => {
                show_images = true;
            }
            "--input" => {
                if i + 1 < args.len() {
                    input = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--model" => {
                if i + 1 < args.len() {
                    model = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    println!("OpenAI CUA Desktop CLI");
    
    // Check for API key
    if env::var("OPENAI_API_KEY").is_err() {
        println!("Error: OPENAI_API_KEY environment variable not set");
        println!("Please set it in your environment or in a .env file");
        return Err(CuaError::Other("OPENAI_API_KEY not set".to_string()));
    }
    
    // Create API client with the specified model
    let client = OpenAIClient::from_env(model)?;
    
    // Create computer
    let computer: Box<dyn Computer> = if use_mock {
        println!("Using mock computer implementation");
        Box::new(MockComputer::new("linux", 1920, 1080))
    } else {
        println!("Using thread-based desktop implementation");
        match ThreadComputer::new() {
            Ok(computer) => Box::new(computer),
            Err(e) => {
                println!("Error creating thread-based computer: {}", e);
                println!("Falling back to mock implementation");
                Box::new(MockComputer::new("linux", 1920, 1080))
            }
        }
    };
    
    println!("Computer environment: {}", computer.environment());
    println!("Screen dimensions: {:?}", computer.dimensions());
    
    // Create safety check callback
    let safety_check: SafetyCheckCallback = Box::new(|message| {
        println!("Safety Check: {}", message);
        print!("Do you want to allow this action? (y/n): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        input.trim().to_lowercase() == "y"
    });
    
    // Create agent
    let agent = Agent::new(
        client,
        computer,
        Vec::new(),
        Some(safety_check),
    )
    .with_debug(debug)
    .with_show_images(show_images);
    
    // Run the agent
    if let Some(initial_input) = input {
        println!("Running with initial input: {}", initial_input);
        let _ = agent.run(&initial_input).await?;
    }
    
    // Run interactively
    agent.run_interactive().await?;
    
    Ok(())
}