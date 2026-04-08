//! Hello World Skill - A simple example skill for CEOClaw
//!
//! This skill demonstrates the basic structure of a CEOClaw skill.

use ceo_claw_sdk::prelude::*;

/// Main skill entry point
#[skill_main]
fn hello_world_skill(context: &SkillContext, args: SkillArgs) -> SkillResult {
    log_info!("Hello World skill started");

    // Get the name argument, or use a default
    let name = args.get_string_or_default("name", "World".to_string());

    // Create the greeting
    let greeting = format!("Hello, {}! from CEOClaw skill", name);

    log_info!("Skill executed successfully");

    // Return the response
    Ok(SkillResponse::success(WasmArgument::string(greeting)))
}