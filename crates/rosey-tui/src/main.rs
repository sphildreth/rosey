use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("warn").init();

    println!("Rosey TUI starter");
    println!();
    println!("This crate will become the Ratatui/Crossterm interface.");
    println!("Build the core + CLI parity harness first, then wire this UI to engine events.");
    println!();
    println!("Planned screens:");
    println!("  1. Dashboard");
    println!("  2. Scan Results");
    println!("  3. Plan Preview");
    println!("  4. Transfer Queue");
    println!("  5. Logs / Recovery");
    println!("  6. Settings");

    Ok(())
}
