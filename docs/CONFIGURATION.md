# Configuration

Rosey Rust will use a user-editable config file and avoid storing secrets directly in plain text.

Planned config shape:

```toml
[source]
default_root = "/mnt/incoming"

[target]
movies = "/mnt/media/Movies"
tv = "/mnt/media/TV"

[scan]
follow_symlinks = false
max_parallel_local = 8
max_parallel_network = 3

[moves]
default_mode = "dry_run"
conflict_policy = "skip"
verify_cross_volume_copies = true
journal_enabled = true

[metadata]
online_lookups = false
language = "en-US"
region = "US"
```

Configuration format is subject to change during the migration.
