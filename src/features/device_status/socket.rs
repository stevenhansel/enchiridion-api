use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use rand::{rngs::StdRng, Rng};
use rand_core::SeedableRng;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StatusMessage(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<StatusMessage>,
    pub device_id: i32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Debug)]
pub struct StatusSocketServer {
    sessions: Arc<Mutex<HashMap<usize, Recipient<StatusMessage>>>>,
    devices: Arc<Mutex<HashMap<i32, HashSet<usize>>>>,
    rng: StdRng,
}

impl StatusSocketServer {
    pub fn new(
        sessions: Arc<Mutex<HashMap<usize, Recipient<StatusMessage>>>>,
        devices: Arc<Mutex<HashMap<i32, HashSet<usize>>>>,
    ) -> Self {
        StatusSocketServer {
            sessions,
            devices,
            rng: SeedableRng::from_entropy(),
        }
    }
}

impl Actor for StatusSocketServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for StatusSocketServer {
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

impl Handler<Disconnect> for StatusSocketServer {
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
