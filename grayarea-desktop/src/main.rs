#![allow(clippy::unnecessary_mut_passed)]

use structopt::StructOpt;
use grayarea_desktop::Opt;
use tokio::process::Command;
use orchestrator::{Orchestrator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_log_engine();
    let opt = Opt::from_args();
    let config = opt.load_config().await?;    

    // Start out commands
    let mut orchestrator = Orchestrator::default()
        .ipc(true)
        .rust_backtrace(opt.debug);
    for stage in config.functions.iter() {
        let mut cmd = if opt.debug {
                let mut cmd = Command::new("cargo");
                cmd.arg("run").arg("--package=grayarea-runtime");
                cmd
            } else {
                Command::new("grayarea-runtime")
            };
        
        orchestrator.start(&stage.name, cmd.arg(&stage.config))
            .expect("failed to start process");
    };

    // Estiblish connections between commands
    let mut orchestra = orchestrator.connect().await?;
    for i in 0..orchestra.bridges.len()-1 {
        orchestra.pipe_bridges(i, i+1)?;
    }

    // Killing it hard since some spawned futures might still run
    match orchestra.run().await {
        Err(_) => std::process::exit(1),
        _ => Ok(())
    }
}

fn init_log_engine() {
    let mut builder = pretty_env_logger::formatted_timed_builder();
    builder.filter_level(log::LevelFilter::Info)
        .default_format_module_path(true);
    builder.init();
}
