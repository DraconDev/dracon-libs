# dracon-system

System diagnostics, SSH remote execution, and desktop notifications.

## Usage

```rust
use dracon_system::{SystemSnapshotProvider, SshRemoteConnector};

// Local system info
let provider = SystemSnapshotProvider::new()?;
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

None (uses `sysinfo`, `ssh2`, `notify-rust`).

## License

MIT OR Apache-2.0
