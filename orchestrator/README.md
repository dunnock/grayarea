Execute and orchestrate command line utils.

Based on io streams and optionally ipc-channel orchestrator is intended for starting and orchestrating programs.


# Use case
```
use tokio::process::{Command};
use orchestrator::Orchestrator;
let mut orchestrator = Orchestrator::default().ipc(false);
orchestrator.start("start", &mut Command::new("echo"));
orchestrator.connect();
```