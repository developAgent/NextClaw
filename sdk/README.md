# CEOClaw Skill Development SDK

The CEOClaw SDK provides everything you need to create WASM-based skills for CEOClaw.

## Quick Start

### Prerequisites

- Rust 1.70+ with WASM target
- wasm-pack

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### Creating a Skill

1. Create a new Rust library project:

```bash
cargo new --lib my-skill
cd my-skill
```

2. Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
ceo-claw-sdk = "0.1.0"
```

3. Implement your skill in `src/lib.rs`:

```rust
use ceo_claw_sdk::prelude::*;

#[skill_main]
fn my_skill(context: &SkillContext, args: SkillArgs) -> SkillResult {
    let name = args.get_string_or_default("name", "World".to_string());
    let greeting = format!("Hello, {}!", name);

    Ok(SkillResponse::success(WasmArgument::string(greeting)))
}
```

4. Create a `manifest.json`:

```json
{
  "id": "com.example.my-skill",
  "name": "My Skill",
  "version": "1.0.0",
  "description": "A sample skill",
  "author": "Your Name",
  "api_version": "1.0",
  "entry_point": "main",
  "permissions": []
}
```

5. Build your skill:

```bash
wasm-pack build --target web --release
```

## Skill API

### Context

The `SkillContext` provides information about the execution environment:

```rust
let execution_id = context.execution_id();
let skill_id = context.skill_id();
let skill_version = context.skill_version();
```

### Arguments

Access skill arguments:

```rust
let value = args.get_string("key")?;
let number = args.get_number("count")?;
let flag = args.get_bool("enabled")?;
```

### Response

Return a response:

```rust
// Success with data
Ok(SkillResponse::success("Result"))

// Error
Err(SkillError::missing_argument("key"))

// Error with code
Err(SkillError::invalid_argument_value("key", "must be positive"))
```

### Logging

Log messages from your skill:

```rust
log_info!("Processing request");
log_debug!("Debug information");
log_warn!("Warning message");
log_error!("Error occurred");
```

## Permissions

Skills require permissions to access certain resources. Specify them in your manifest:

```json
{
  "permissions": [
    {
      "permission_type": "file.read",
      "description": "Read files from filesystem",
      "scope": "/tmp",
      "required": true
    }
  ]
}
```

Available permission types:
- `file.read` / `file.write` / `file.delete` / `file.execute`
- `network.http` / `network.ws` / `network.tcp` / `network.udp`
- `system.exec` / `system.env` / `system.process`
- `clipboard.read` / `clipboard.write`
- `notification.send`
- `database.read` / `database.write`

## Examples

See the `examples/` directory for complete skill examples:

- **hello-world**: Simple greeting skill
- **file-organizer**: File organization with permissions
- **web-fetcher**: HTTP requests and JSON parsing

## Installation

1. Build your skill: `wasm-pack build --target web --release`
2. Open CEOClaw Settings > Skills
3. Click "Install Skill" and select your `manifest.json`

## Testing

Test your skill locally:

```bash
# Run tests
cargo test

# Build and test with wasm-pack
wasm-pack test --node
```

## Distribution

Package your skill for distribution:

```bash
# Create a distribution package
mkdir dist
cp manifest.json dist/
cp pkg/my_skill_bg.wasm dist/
cp pkg/my_skill.js dist/
tar czf my-skill.tar.gz dist/
```

## Advanced Topics

### Async Skills

For long-running operations, use async:

```rust
#[skill_main]
fn async_skill(context: &SkillContext, args: SkillArgs) -> SkillResult {
    // Your async code here
    Ok(SkillResponse::success("Done"))
}
```

### Error Handling

Custom error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MySkillError {
    #[error("Invalid input")]
    InvalidInput,
}

impl From<MySkillError> for SkillError {
    fn from(e: MySkillError) -> Self {
        SkillError::Other(e.to_string())
    }
}
```

### Metadata

Add metadata to responses:

```rust
Ok(SkillResponse::success(data)
    .with_metadata("version", "1.0".to_string())
    .with_metadata("timestamp", chrono::Utc::now().to_rfc3339()))
```

## Support

- Documentation: https://docs.ceoclaw.com/sdk
- GitHub Issues: https://github.com/ceoclaw/ceo-claw-sdk/issues
- Discord: https://discord.gg/ceoclaw

## License

MIT License - see LICENSE file for details