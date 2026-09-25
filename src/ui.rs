use std::{error::Error as _, process::exit, sync::Arc};

use iced::{
    Element, Length, Subscription, Task, Theme,
    stream::channel,
    widget::{button, column, row, space, text_input},
};
use indented::indented;
use inline_colorization::{
    color_bright_cyan, color_bright_red, color_bright_white, color_white, style_bold, style_reset,
};
use lucide_iced::icon::cloud_download;

use self::calendar::Calendar;
use crate::{
    config::Config,
    error::{Error, Result},
};

mod calendar;
mod email;

#[derive(Debug, Clone)]
pub enum OutMessage<In, Up> {
    UpMessage(Up),
    InMessage(In),
}

#[derive(Debug, Clone)]
pub enum Message {
    Startup,
    // UnifiedPushEvent(Box<PushEvent>),
    NewEndpoint { url: String },
    MessageReceived,
    Unregistered,
    UnifiedPushError(String),
    StartSync,
    SyncFinished { token: String },
    Calendar(calendar::InMessage),
    Email(email::InMessage),
    SearchChanged(String),
    LethalError(Arc<Error>),
    Nothing,
}

impl From<calendar::OutMessage> for Message {
    fn from(value: calendar::OutMessage) -> Self {
        use OutMessage::*;
        match value {
            UpMessage(calendar::UpMessage::Error(error)) => Self::LethalError(error),
            UpMessage(calendar::UpMessage::Nothing) => Self::Nothing,
            InMessage(msg) => Self::Calendar(msg),
        }
    }
}
impl From<email::OutMessage> for Message {
    fn from(msg: email::OutMessage) -> Self {
        match msg {
            OutMessage::InMessage(msg) => Self::Email(msg),
        }
    }
}

impl From<()> for Message {
    fn from((): ()) -> Self {
        Self::Nothing
    }
}

impl<T: Into<Message>> From<Result<T>> for Message {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(m) => m.into(),
            Err(error) => Self::LethalError(Arc::new(error)),
        }
    }
}

#[derive(Debug)]
pub struct App {
    calendar: Calendar,
    #[allow(unused)]
    theme: Theme,
    search: String,
    syncing: bool,
    config: Config,
}

impl App {
    pub const APP_NAME: &str = "ad.2mon.ems.calendar";

    pub fn new() -> (Self, Task<Message>) {
        let config = Config::new().unwrap();
        let (calendar, task) = Calendar::new(Self::APP_NAME, &config.calendar);

        (
            Self {
                calendar,
                theme: Theme::Dark,
                search: String::new(),
                storage: StorageStatus::Starting,
                syncing: false,
                config,
            },
            Task::batch([Task::done(Message::Startup), task.map(Into::into)]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Startup => {
                // let stream = stream! {
                //     let storage = UnifiedPushStoragePreferences::new(AppInfo {
                //         name: Self::APP_NAME,
                //         author: "jthulhu",
                //     });
                //     let (sender, mut receiver) = unbounded_channel::<PushEvent>();

                //     let handle = Handle::try_current()
                //     .or_else(|_| Ok::<_, std::io::Error>(Runtime::new()?.handle().to_owned()))
                //     .map_err(|error| error.to_string())?;

                //     let unified_push = spawn_local(async move {
                //         UnifiedPush::new(Self::APP_NAME, storage, sender, handle)
                //         .await
                //         .map_err(|error| error.to_string())
                //     }).await.unwrap()?;

                //     if !spawn_local(async move {
                //         unified_push.try_use_default_distributor().await
                //     }).await.unwrap() {
                //         panic!()
                //     };{}
                //     while let Some(curr) = receiver.recv().await {
                //         yield Ok(curr);
                //     }
                // };
                // Task::stream(stream.map(|res: Result<_, String>| match res {
                //     Ok(event) => Message::UnifiedPushEvent(Box::new(event)),
                //     Err(error) => Message::UnifiedPushError(error),
                // }))
                Task::none()
            }
            // Message::UnifiedPushEvent(push_event) => todo!(),
            Message::UnifiedPushError(_) => todo!(),
            Message::Calendar(cal_msg) => self.calendar.update(cal_msg).map(Into::into),
            Message::StartSync => {
                if let StorageStatus::ConnectionEstablished(ref storage) = self.storage
                    && !self.syncing
                {
                    self.syncing = true;
                    Task::perform(
                        {
                            let storage = storage.clone();
                            async move { storage.sync_calendars().await }
                        },
                        Into::into,
                    )
                    .chain(Task::perform(
                        {
                            let storage = storage.clone();
                            async move { storage.sync_calendars().await }
                        },
                        Into::into,
                    ))
                } else {
                    Task::none()
                }
            }
            Message::SyncFinished { token } => {
                self.syncing = false;
                Task::none()
            }
            Message::SearchChanged(s) => {
                self.search = s;
                Task::none()
            }
            Message::NewEndpoint { url } => todo!(),
            Message::MessageReceived => todo!(),
            Message::Unregistered => todo!(),
            Message::LethalError(error) => {
                eprintln!(
                    "{style_bold}{color_bright_red}error{color_bright_white}: {error}{style_reset}{color_white}"
                );
                if let Some(ref info) = error.info {
                    eprintln!(
                        "  {color_bright_cyan}-->{color_white} {}:{}:{}",
                        info.file(),
                        info.line(),
                        info.column()
                    );
                    // eprintln!(" Backtrace:");
                    // eprintln!("{}", indented(info.backtrace()));
                }
                let mut source = error.source();
                while let Some(err) = source {
                    eprintln!("Caused by:");
                    eprintln!("{}", indented(err));
                    source = err.source();
                }
                exit(1)
            }
            Message::Nothing => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            row![
                space().width(Length::Fill),
                text_input("Search...", &self.search)
                    .on_input(Message::SearchChanged)
                    .width(Length::FillPortion(3)),
                space().width(Length::Fill),
                button(cloud_download())
                    .on_press_maybe(if self.syncing {
                        None
                    } else {
                        Some(Message::StartSync)
                    })
                    .padding(5.0)
                    .width(Length::Shrink),
            ]
            .padding(5.0)
            .height(Length::Fixed(50.0)),
            row![self.calendar.view().map(Into::into)]
        ]
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            channel(
                100,
                #[allow(unused)]
                |mut output| async {
                    // let storage = UnifiedPushStoragePreferences
                },
            )
        })
    }
}
