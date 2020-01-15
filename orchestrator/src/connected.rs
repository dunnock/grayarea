use crate::message::Message;
use crate::should_not_complete;
use crate::{Bridge, Receiver, Sender};
use anyhow::anyhow;
use crossbeam::channel;
use futures::future::{try_join_all, FusedFuture, FutureExt};
use futures::{pin_mut, select};
use log::{error, info};
use std::collections::HashMap;
use std::pin::Pin;
use tokio::task::JoinHandle;
use ipc_channel::ipc::{IpcReceiverSet, IpcSelectionResult};

type TryAllPin = Pin<Box<dyn FusedFuture<Output = anyhow::Result<Vec<()>>>>>;

/// Orchestrator with successfully started processes connected via IPC
pub struct ConnectedOrchestrator<LF: FusedFuture> {
    pub bridges: HashMap<String, Bridge>,
    routes: Option<HashMap<String, Vec<Sender>>>,
    pipes: Vec<JoinHandle<anyhow::Result<()>>>,
    loggers: Pin<Box<LF>>,
    processes: TryAllPin,
}

impl<LF> ConnectedOrchestrator<LF>
where
    LF: FusedFuture<Output = anyhow::Result<Vec<()>>>,
{
    pub(crate) fn new(bridges: Vec<Bridge>, processes: TryAllPin, loggers: Pin<Box<LF>>) -> Self {
        ConnectedOrchestrator {
            bridges: bridges
                .into_iter()
                .map(|bridge| (bridge.name.clone(), bridge))
                .collect(),
            routes: Some(HashMap::new()),
            processes,
            loggers,
            pipes: Vec::new(),
        }
    }

    /// Build a pipe from modules b_in to b_out
    /// Spawns pipe handler in a tokio blocking task thread
    /// - b_in name of incoming bridge from Self::bridges
    /// - b_out name of outgoing bridge from Self::bridges
    pub fn pipe_bridges(&mut self, b_in: &str, b_out: &str) -> anyhow::Result<()> {
        info!("setting communication {} -> {}", b_in, b_out);
        let rx: Receiver = self.take_bridge_rx(b_in)?;
        let tx: Sender = self.take_bridge_tx(b_out)?;
        let (b_in, b_out) = (b_in.to_owned(), b_out.to_owned());
        let handle = tokio::task::spawn_blocking(move || loop {
            let buf: Message = rx
                .recv()
                .unwrap_or_else(|err| todo!("receiving message from {} failed: {}", b_in, err));
            tx.send(buf)
                .unwrap_or_else(|err| todo!("sending message to {} failed: {}", b_out, err));
        });
        self.pipes.push(handle);
        Ok(())
    }

    /// Forward all messages from module N b_in to crossbeam channel corresponding to topic id
    /// Spawns pipe handler in a tokio blocking task thread
    /// - b_in name of incoming bridge from Self::bridges
    pub fn forward_bridge_rx(
        &mut self,
        b_in: &str,
        out: HashMap<String, channel::Sender<Message>>,
    ) -> anyhow::Result<()> {
        assert!(!out.is_empty());
        info!("setting communication {} -> {} topics", b_in, out.len());
        let rx: Receiver = self.take_bridge_rx(b_in)?;
        let b_in = b_in.to_owned();
        let handle = tokio::task::spawn_blocking(move || loop {
            let msg = rx
                .recv()
                .unwrap_or_else(|err| todo!("receiving message from {} failed: {}", b_in, err));
            assert!(out.contains_key(&msg.topic));
            let topic = msg.topic.clone();
            out[&topic].send(msg).unwrap_or_else(|err| {
                todo!(
                    "sending message from {} to topic {} failed: {}",
                    b_in,
                    topic,
                    err
                )
            });
        });
        self.pipes.push(handle);
        Ok(())
    }

    /// Forward all messages from crossbeam Receiver to module b_out
    /// Spawns pipe handler in a tokio blocking task thread
    /// - b_out name of outgoing bridge from Self::bridges
    pub fn forward_bridge_tx(
        &mut self,
        b_out: &str,
        input: channel::Receiver<Message>,
    ) -> anyhow::Result<()> {
        info!("setting communication topic -> {}", b_out);
        let tx: Sender = self.take_bridge_tx(b_out)?;
        let b_out = b_out.to_owned();
        let handle = tokio::task::spawn_blocking(move || loop {
            let msg: Message = input
                .recv()
                .unwrap_or_else(|err| todo!("receiving message from {} failed: {}", b_out, err));
            tx.send(msg).unwrap_or_else(|err| {
                todo!("sending message from topic to {} failed: {}", b_out, err)
            });
        });
        self.pipes.push(handle);
        Ok(())
    }

    /// Forward all messages received to topic to module bridge b_out
    /// This method only configures route, does not spawn handler. 
    /// After route configuration done handler shall be started with `pipe_routes()`
    /// - topic name of topic for incoming messages
    /// - b_out name of outgoing bridge from Self::bridges
    pub fn route_topic_to_bridge(
        &mut self,
        topic: &str,
        b_out: &str,
    ) -> anyhow::Result<()> {
        info!("setting communication topic {} -> {}", topic, b_out);
        let tx: Sender = self.take_bridge_tx(b_out)?;
        match self.routes.as_mut() {
            Some(r) => match r.get_mut(topic) {
                Some(bridges) => bridges.push(tx) ,
                None => { r.insert(topic.to_owned(), vec![tx]); }
            },
            None => return Err(anyhow::anyhow!("cannot change routes after orchestrator started"))
        };
        Ok(())
    }

    /// Build a pipe from modules b_in to b_out
    /// Spawns pipe handler in a tokio blocking task thread
    /// - b_in name of incoming bridge from Self::bridges
    /// - b_out name of outgoing bridge from Self::bridges
    pub fn pipe_routes(&mut self) -> anyhow::Result<()> {
        info!("starting communication thread");
        let mut ipc_receiver_set = IpcReceiverSet::new().unwrap();
        let mut names: HashMap<u64, String> = HashMap::new();
        let bridge_names: Vec<String> = self.bridges.keys().cloned().collect();
        for name in bridge_names {
            if let Ok(recv) = self.take_bridge_rx(&name) {
                let id = ipc_receiver_set.add(recv)?;
                names.insert(id, name.to_string());
            }
        }
        let routes = self.routes.take()
            .ok_or_else(|| anyhow::anyhow!("routes were not configured"))?;

        let handle = tokio::task::spawn_blocking(move || {
            loop {
                let results = match ipc_receiver_set.select() {
                    Ok(results) => results,
                    Err(err) => todo!("receiving message failed: {}", err),
                };
                for event in results {
                    match event {
                         IpcSelectionResult::MessageReceived(id, message) => {
                            let msg: Message = message.to()
                                .unwrap_or_else(|err| todo!("receiving message from {:?} failed: {}", names.get(&id), err));
                            let senders = routes.get(&msg.topic)
                                .unwrap_or_else(|| todo!("received message from {:?} to topic {} without recepients", names.get(&id), msg.topic));
                            let except_last = senders.len()-1;
                            for (i, tx) in senders[0..except_last].iter().enumerate() {
                                tx.send(msg.clone()).unwrap_or_else(|err|
                                    todo!("sending message from topic {} to {} failed: {}", msg.topic, i, err));
                            };
                            let topic = msg.topic.clone();
                            senders.last().unwrap().send(msg).unwrap_or_else(|err|
                                todo!("sending message from topic {} to last sender failed: {}", topic, err));
                        
                         },
                         IpcSelectionResult::ChannelClosed(id) => {
                             todo!("Channel from {:?} closed...", names.get(&id));
                         }

                    }
                }
            }
        });
        self.pipes.push(handle);
        Ok(())
    }


    /// Run processes to completion
    pub async fn run(self) -> anyhow::Result<()> {
        let pipes = try_join_all(self.pipes).fuse();
        pin_mut!(pipes);
        select!(
            res = pipes => should_not_complete!("channels", res) as anyhow::Result<()>,
            res = self.processes => should_not_complete!("processes", res),
            res = self.loggers => should_not_complete!("logs", res),
        )
    }
}

// Some utilities
impl<LF> ConnectedOrchestrator<LF>
where
    LF: FusedFuture<Output = anyhow::Result<Vec<()>>>,
{
    fn take_bridge_rx(&mut self, name: &str) -> anyhow::Result<Receiver> {
        self.bridges
            .get_mut(name)
            .ok_or_else(|| anyhow!("destination module `{}` bridge not found", name))?
            .channel
            .rx_take()
            .ok_or_else(|| anyhow!("Failed to get receiver from {}", name))
    }

    fn take_bridge_tx(&mut self, name: &str) -> anyhow::Result<Sender> {
        self.bridges
            .get_mut(name)
            .ok_or_else(|| anyhow!("source module `{}` bridge not found", name))?
            .channel
            .tx_take()
            .ok_or_else(|| anyhow!("Failed to get sender from `{}`", name))
    }
}
