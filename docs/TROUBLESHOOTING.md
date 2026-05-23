# Troubleshooting

## Scans are slow

Likely causes:

- network share latency
- antivirus/indexer activity
- very large directory trees
- excessive metadata lookups
- too much parallelism on NAS/NFS/SMB paths

Planned mitigations:

- lower network concurrency
- skip duration probing
- disable online lookups
- use dry-run and inspect logs

## Moves fail

Likely causes:

- destination permissions
- insufficient free space
- path length limits
- destination conflicts
- network share disconnection

Rosey should report the source path, destination path, action, and recovery recommendation.
