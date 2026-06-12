# dracon-system

System diagnostics, SSH remote execution, and desktop notifications.

## Usage

```rust
use dracon_system::{SystemSnapshotProvider, SshRemoteConnector};

// Local system info
let provider = SystemSnapshotProvider::new();
let snap = provider.get_snapshot().await?;
println!("CPU: {}%", snap.cpu_usage);
println!("Memory: {}%", snap.memory_usage);

// Remote execution via SSH
let connector = SshRemoteConnector;
let conn = connector.connect(&request).await?;
let output = conn.exec("htop").await?;
```

## Key Types

- [`SystemSnapshotProvider`] — CPU, memory, disk snapshot
- [`ProcessController`] — process listing and control
- [`SshRemoteConnector`], [`SshRemoteExecProvider`], [`SshRemoteFsProvider`] — SSH remoting
- [`WorkspaceStorageReport`] — workspace disk usage analysis

## Feature Flags

- `unsafe-remote-shell` — opts into the deprecated raw remote shell execution API. Prefer structured `exec_program()` calls for remote execution.

## License

AGPL-3.0-only
