#![allow(clippy::unnecessary_mut_passed)]

use crossbeam::channel;
use futures::future::try_join_all;
use grayarea_desktop::Opt;
use orchestrator::{message::Message, orchestrator};
use std::collections::HashMap;
use structopt::StructOpt;
use tokio::process::Command;

const CHANNEL_SIZE: usize = 10;

type TopicChannel = (channel::Sender<Message>, channel::Receiver<Message>);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_log_engine();
    let opt = Opt::from_args();
    let config = opt.load_config().await?;
    // Load all modules configs
    let modules = try_join_all(config.functions.iter().map(|module| module.load_config())).await?;
    // Create crossbeam channels for all input topics
    let in_topics: HashMap<String, TopicChannel> = modules
        .iter()
        .filter_map(|module| {
            module
                .input
                .as_ref()
                .map(|input| (input.topic.clone(), channel::bounded(CHANNEL_SIZE)))
        })
        .collect();

    // Start out commands
    let mut orchestrator = orchestrator().ipc(true).rust_backtrace(opt.debug);
    for stage in config.functions.iter() {
        let mut cmd = if opt.debug {
            let mut cmd = Command::new("cargo");
            cmd.arg("run").arg("--package=grayarea-runtime");
            cmd
        } else {
            Command::new("grayarea-runtime")
        };

        orchestrator
            .start(&stage.name, cmd.arg(&stage.config))
            .expect("failed to start process");
    }

    // Estiblish connections between commands
    let mut orchestra = orchestrator.connect().await?;
    for (i, module) in modules.into_iter().enumerate() {
        // Connect module's outputs to relevant topics
        // Will fail if some output topics have not relevant sinks
        if let Ok(topics) = module.topics() {
            let mut out_topics = Vec::with_capacity(topics.len());
            for name in topics {
                let topic = in_topics.get(&name).ok_or_else(|| {
                    anyhow::anyhow!("topic {} is not pointing to any function input", name)
                })?;
                out_topics.push(topic.0.clone());
            }
            orchestra.forward_bridge_rx(i, out_topics)?;
        }
        // Connect module's input to topic
        if let Some(Some(topic)) = module
            .input
            .as_ref()
            .map(|input| in_topics.get(&input.topic))
        {
            orchestra.forward_bridge_tx(i, topic.1.clone())?;
        }
    }

    // Killing it hard since some spawned futures might still run
    match orchestra.run().await {
        Err(_) => std::process::exit(1),
        _ => Ok(()),
    }
}

fn init_log_engine() {
    let mut builder = pretty_env_logger::formatted_timed_builder();
    builder
        .filter_level(log::LevelFilter::Info)
        .default_format_module_path(true);
    builder.init();
}
