use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::Message;

pub struct Payload {
    pub(crate) message: Message,
    pub(crate) sender: oneshot::Sender<Option<Vec<u8>>>,
}

impl Payload {
    pub fn new(message: Message) -> (oneshot::Receiver<Option<Vec<u8>>>, Self) {
        let (sender, receiver) = oneshot::channel();
        (receiver, Self { message, sender })
    }
}
