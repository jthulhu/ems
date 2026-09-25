use std::sync::Arc;

use derive_more::From;
use iced::{Element, Task};

use crate::{
    config::email::EmailConfig,
    db::email::EmailStorage,
    error::{Error, Result},
};

#[derive(Debug, Clone)]
pub enum UpMessage {
    Error(Arc<Error>),
}

#[derive(Debug, Clone, From)]
pub enum InMessage {
    OpenStorage(EmailStorage),
}

pub type OutMessage = super::OutMessage<InMessage, UpMessage>;

impl<T> From<Result<T>> for OutMessage
where
    T: Into<InMessage>,
{
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(msg) => OutMessage::InMessage(msg.into()),
            Err(error) => OutMessage::UpMessage(UpMessage::Error(Arc::new(error))),
        }
    }
}

#[derive(Debug)]
pub struct Email {
    storage: Option<EmailStorage>,
}

impl Email {
    pub fn new(app_name: &'static str, config: &EmailConfig) -> (Self, Task<OutMessage>) {
        let storage = EmailStorage::open(config.database_path.clone(), app_name);
        (
            Self { storage: None },
            Task::batch([Task::perform(storage, Into::into)]),
        )
    }

    pub fn view(&self) -> Element<'_, OutMessage> {
        todo!()
    }

    pub fn update(&mut self, message: InMessage) -> Task<OutMessage> {
        match message {
            InMessage::OpenStorage(storage) => {
                self.storage = Some(storage);
                Task::none()
            }
        }
    }
}
