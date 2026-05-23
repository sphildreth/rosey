# Prompt: Build Ratatui/Crossterm TUI

Build the Rosey TUI after the core and CLI are useful.

## Target crate

- `crates/rosey-tui`

## Requirements

Screens:

- Dashboard
- Scan Results
- Plan Preview
- Transfer Queue
- Logs / Recovery
- Settings
- Help

Keyboard commands:

```text
s     scan
p     preview plan
d     dry-run
m     execute move/copy
c     conflict policy
/     filter/search
tab   next panel
?     help
q     quit
```

## Architecture

The TUI must render app state and send commands. It must not contain scanner/planner/mover business logic.

Use engine events from core/fs crates.

## Success criteria

- TUI starts cleanly
- no panic on terminal resize
- help screen documents keys
- dry-run before execute
- destructive execution requires confirmation
