extern crate actor;

use actor::*;
use tokio::sync::{mpsc, oneshot};

// #[gen_actor(AddMsg, RemoveMsg)]
struct MyActor(i64);

#[async_trait::async_trait]
impl Actor for MyActor {
    type Msg = MyActorMsg;

    async fn handle(&mut self, msg: Self::Msg) {
        self.dispatch(msg).await;
    }
}

async fn start_actor(mut actor: MyActor, mut receiver: mpsc::Receiver<MyActorMsg>) {
    println!("actor starting");
    while let Some(msg) = receiver.recv().await {
        actor.handle(msg).await;
    }
    println!("actor ended");
}

// generate

enum MyActorMsg {
    AddMsg(AddMsg),
    RemoveMsg(RemoveMsg),
    GetMsg(Get),
}

impl MyActor {
    async fn dispatch(&mut self, msg: MyActorMsg) {
        match msg {
            MyActorMsg::AddMsg(msg) => self.receive(msg).await,
            MyActorMsg::RemoveMsg(msg) => self.receive(msg).await,
            MyActorMsg::GetMsg(msg) => self.receive(msg).await,
        }
    }
}

// Add
struct AddMsg {
    value: i64,
}

impl From<AddMsg> for MyActorMsg {
    fn from(msg: AddMsg) -> Self {
        Self::AddMsg(msg)
    }
}

#[async_trait::async_trait]
impl Receive<AddMsg> for MyActor {
    type Msg = MyActorMsg;

    async fn receive(&mut self, msg: AddMsg) {
        self.0 += msg.value;
    }
}

// Remove

struct RemoveMsg {
    value: i64,
}

impl From<RemoveMsg> for MyActorMsg {
    fn from(msg: RemoveMsg) -> Self {
        Self::RemoveMsg(msg)
    }
}

#[async_trait::async_trait]
impl Receive<RemoveMsg> for MyActor {
    type Msg = MyActorMsg;

    async fn receive(&mut self, msg: RemoveMsg) {
        self.0 -= msg.value;
    }
}

// Get

struct Get {
    sender: oneshot::Sender<i64>,
}

impl From<Get> for MyActorMsg {
    fn from(msg: Get) -> Self {
        Self::GetMsg(msg)
    }
}

#[async_trait::async_trait]
impl Receive<Get> for MyActor {
    type Msg = MyActorMsg;

    async fn receive(&mut self, msg: Get) {
        let _ = msg.sender.send(self.0);
    }
}

#[derive(Clone)]
pub struct Client {
    sender: mpsc::Sender<MyActorMsg>,
}

impl Client {
    pub fn new(chan_size: usize) -> Self {
        let (sender, receiver) = mpsc::channel(chan_size);
        let actor = MyActor(0);

        println!("starting new actor");
        tokio::spawn(start_actor(actor, receiver));
        Self { sender }
    }

    pub async fn add(&self, v: i64) {
        let msg = AddMsg { value: v };
        let _ = self.sender.send(msg.into()).await;
    }

    pub async fn rem(&self, v: i64) {
        let msg = RemoveMsg { value: v };
        let _ = self.sender.send(msg.into()).await;
    }

    pub async fn get(&self) -> i64 {
        let (sender, rx) = oneshot::channel();
        let msg = Get { sender };
        let _ = self.sender.send(msg.into()).await;
        rx.await.expect("get value")
    }
}

#[tokio::main]
async fn main() {
    let a = Client::new(10);
    a.add(10).await;
    a.rem(2).await;

    let v = a.get().await;
    println!("value is {}", v);

    a.add(34).await;
    let v = a.get().await;
    println!("value is {}", v);
}
