# Golden Outputs

This folder stores normalized expected JSON outputs, preferably generated from the existing Python Rosey implementation.

Golden files should be deterministic and avoid unstable data such as:

- absolute temp paths
- timestamps
- provider responses
- unordered maps
- OS-specific path separators when possible
