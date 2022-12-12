use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use rand::{rngs::StdRng, Rng};
use rand_core::SeedableRng;

#[derive(Message)]
#[rtype(result = "()")]
pub struct LivestreamMessage(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<LivestreamMessage>,
    pub device_id: i32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Debug)]
pub struct LivestreamSocketServer {
    sessions: Arc<Mutex<HashMap<usize, Recipient<LivestreamMessage>>>>,
    devices: Arc<Mutex<HashMap<i32, HashSet<usize>>>>,
    rng: StdRng,
}

impl LivestreamSocketServer {
    pub fn new(
        sessions: Arc<Mutex<HashMap<usize, Recipient<LivestreamMessage>>>>,
        devices: Arc<Mutex<HashMap<i32, HashSet<usize>>>>,
    ) -> Self {
        LivestreamSocketServer {
            sessions,
            devices,
            rng: SeedableRng::from_entropy(),
        }
    }
}

impl Actor for LivestreamSocketServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for LivestreamSocketServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let mut sessions = self.sessions.lock().unwrap();
        let mut devices = self.devices.lock().unwrap();

        let id = self.rng.gen::<usize>();
        sessions.insert(id, msg.addr);

        devices
            .entry(msg.device_id)
            .or_insert_with(HashSet::new)
            .insert(id);

        id
    }
}

impl Handler<Disconnect> for LivestreamSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let mut sessions = self.sessions.lock().unwrap();

        if sessions.remove(&msg.id).is_some() {
            let mut devices = self.devices.lock().unwrap();
            for (_, sessions) in devices.iter_mut() {
                sessions.remove(&msg.id);
            }
        }
    }
}
