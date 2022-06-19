use tokio::sync::mpsc::{self, error::SendError};

// pub use actor_macros::gen_actor;

pub trait Message: Send + 'static {}

impl<T: Send + 'static> Message for T {}

#[async_trait::async_trait]
pub trait Receive<Msg: Message> {
    type Msg: Message;

    async fn receive(&mut self, msg: Msg);
}

#[async_trait::async_trait]
pub trait Actor: Send + 'static {
    type Msg: Message;

    async fn handle(&mut self, msg: Self::Msg);
}

pub struct Address<Msg>(mpsc::Sender<Msg>);

impl<Msg> Address<Msg> {
    pub fn new(size: usize) -> (Self, mpsc::Receiver<Msg>) {
        let (tx, rx) = mpsc::channel(size);
        (Self(tx), rx)
    }

    // TODO return an error
    pub async fn send<T>(&self, msg: T) -> Result<(), SendError<Msg>>
    where
        T: Into<Msg>,
    {
        self.0.send(msg.into()).await
    }
}
