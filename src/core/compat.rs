use bevy::prelude::{Message, MessageWriter};

pub trait MessageWriterSendExt<M: Message> {
    fn send(&mut self, message: M);
}

impl<'w, M: Message> MessageWriterSendExt<M> for MessageWriter<'w, M> {
    fn send(&mut self, message: M) {
        self.write(message);
    }
}
