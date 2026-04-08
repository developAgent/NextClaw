//! CEOClaw SDK for WASM skill development
//!
//! This SDK provides the tools and types needed to develop WASM skills for CEOClaw.
//!
//! # Example
//!
//! ```no_run
//! use ceo_claw_sdk::prelude::*;
//!
//! #[skill_main]
//! fn my_skill(context: &SkillContext, args: SkillArgs) -> SkillResult {
//!     let input = args.get_string("input")?;
//!     let response = format!("Hello, {}!", input);
//!     Ok(SkillResponse::success(response))
//! }
//! ```

pub mod context;
pub mod types;
pub mod utils;
pub mod manifest;

// Prelude module for convenience
pub mod prelude {
    pub use crate::context::{SkillContext, ContextBuilder};
    pub use crate::types::{
        SkillArgs, SkillResult, SkillResponse, SkillError,
        WasmArgument, WasmArgumentType, ErrorCode,
    };
    pub use crate::manifest::{
        SkillManifest, SkillManifestBuilder,
        SkillPermission, PermissionBuilder,
    };
    pub use crate::utils::{
        log, log_debug, log_info, log_warn, log_error,
    };
}

pub use prelude::*;

/// Macro to mark the main entry point of a skill
#[macro_export]
macro_rules! skill_main {
    ($fn_name:ident) => {
        #[no_mangle]
        pub extern "C" fn main() -> i32 {
            use $crate::prelude::*;
            use std::panic;

            // Set up panic handler
            panic::set_hook(Box::new(|panic_info| {
                log_error(&format!("Skill panic: {}", panic_info));
            }));

            // Create context
            let context = SkillContext::new();

            // Parse arguments from environment
            let args = SkillArgs::from_env();

            // Call the main function
            let result = panic::catch_unwind(|| {
                $fn_name(&context, args)
            });

            match result {
                Ok(Ok(response)) => {
                    // Output the response
                    println!("{}", serde_json::to_string(&response).unwrap());
                    0
                }
                Ok(Err(e)) => {
                    log_error(&format!("Skill error: {}", e));
                    e.code() as i32
                }
                Err(e) => {
                    log_error(&format!("Skill panic: {:?}", e));
                    1
                }
            }
        }
    };
}

/// Re-export commonly used types
pub use context::*;
pub use types::*;
pub use manifest::*;
pub use utils::*;