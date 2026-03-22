use am_core::output::Format;

const REPO: &str = "https://github.com/Sortis-AI/am";

const INSTRUCTIONS: &str = r#"# Install the am agent skill

The am skill teaches AI agents how to use Agent Messenger for E2E
encrypted communication over Nostr.

## Claude Code

Add to your project's .mcp.json or install globally:

    claude mcp add-skill https://github.com/Sortis-AI/am

Or clone and point directly:

    git clone https://github.com/Sortis-AI/am.git
    claude mcp add-skill ./am

## Other agents (vercel-labs/skills compatible)

Clone the repository and point your agent at the skills/ directory:

    git clone https://github.com/Sortis-AI/am.git

The skill is at: am/skills/am/SKILL.md

## What the skill covers

- Identity management, relay configuration, first-time setup
- Sending and receiving encrypted 1:1 and group messages
- Profile metadata publishing
- JSON output parsing and exit codes
- Agent harness (am-ingest + am-agent) for autonomous operation
"#;

pub fn print_instructions(format: Format) {
    match format {
        Format::Json => {
            let json = serde_json::json!({
                "repository": REPO,
                "skill_path": "skills/am/SKILL.md",
                "install_claude_code": format!("claude mcp add-skill {REPO}"),
                "install_manual": format!("git clone {REPO}.git"),
                "instructions": INSTRUCTIONS.trim(),
            });
            let _ = am_core::output::print_json(&json);
        }
        Format::Text => {
            print!("{INSTRUCTIONS}");
        }
    }
}
