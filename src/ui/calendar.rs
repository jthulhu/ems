use chrono::{Local, NaiveDate};
use day_view::DayView;
use derive_more::From;
use iced::{
    Element, Length, Task,
    widget::{column, row},
};
use infinite_view::InfiniteView;
use moka::sync::Cache;
use month_view::MonthView;

use std::sync::Arc;

use crate::{
    config::CalendarConfig,
    db::calendar::{CalendarStorage, Occurrence},
    error::{Error, Result},
};

mod day_view;
mod infinite_view;
mod month_view;

pub type OutMessage = super::OutMessage<InMessage, UpMessage>;

#[derive(Clone, Debug, From)]
pub enum InMessage {
    InfiniteMsg(infinite_view::InMessage),
    DayMsg(day_view::InMessage),
    MonthMsg(month_view::InMessage),
    #[from(skip)]
    Select(NaiveDate),
    OpenStorage(CalendarStorage),
}

#[derive(Debug, Clone)]
pub enum UpMessage {
    Error(Arc<Error>),
    Nothing,
}

impl From<day_view::OutMessage> for OutMessage {
    fn from(msg: day_view::OutMessage) -> Self {
        use super::OutMessage;

        match msg {
            OutMessage::InMessage(msg) => OutMessage::InMessage(InMessage::DayMsg(msg)),
        }
    }
}

impl From<month_view::OutMessage> for OutMessage {
    fn from(msg: month_view::OutMessage) -> Self {
        use super::OutMessage;

        match msg {
            OutMessage::UpMessage(month_view::UpMessage::Select(date)) => {
                OutMessage::InMessage(InMessage::Select(date))
            }
        }
    }
}

impl From<infinite_view::OutMessage> for OutMessage {
    fn from(msg: infinite_view::OutMessage) -> Self {
        use super::OutMessage;

        match msg {
            OutMessage::UpMessage(infinite_view::UpMessage::Error(error)) => {
                OutMessage::UpMessage(UpMessage::Error(error))
            }
            OutMessage::UpMessage(infinite_view::UpMessage::Select(date)) => {
                OutMessage::InMessage(InMessage::Select(date))
            }
            OutMessage::InMessage(msg) => OutMessage::InMessage(InMessage::InfiniteMsg(msg)),
        }
    }
}

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
pub struct Calendar {
    cache: Cache<NaiveDate, Vec<Occurrence>>,
    inf_view: InfiniteView,
    day_view: DayView,
    month_view: MonthView,
    selected: NaiveDate,
    storage: Option<CalendarStorage>,
}

impl Calendar {
    pub fn new(app_name: &str, config: &CalendarConfig) -> (Self, Task<OutMessage>) {
        let today = Local::now().date_naive();
        let cache = Cache::new(1000);
        let storage = CalendarStorage::open(config.database_path.clone(), app_name);
        let (inf_view, inf_task) = InfiniteView::new(cache.clone());
        let (day_view, day_task) = DayView::new(cache.clone());
        let (month_view, month_task) = MonthView::new();
        (
            Self {
                inf_view,
                day_view,
                month_view,
                cache,
                selected: today,
                storage: None,
            },
            Task::batch([
                inf_task.map(|msg| match msg {}),
                day_task.map(|msg| match msg {}),
                month_task.map(|msg| match msg {}),
                Task::perform(storage, Into::into),
            ]),
        )
    }

    pub fn view(&self) -> Element<'_, OutMessage> {
        row![
            self.inf_view.view(self.selected).map(Into::into),
            column![
                self.day_view.view(self.selected).map(Into::into),
                self.month_view.view(self.selected).map(Into::into),
            ]
            .height(Length::Fill)
            .width(Length::FillPortion(1))
            .spacing(10)
            .padding(10),
        ]
        .into()
    }

    pub fn update(&mut self, message: InMessage) -> Task<OutMessage> {
        match message {
            InMessage::Select(date) => {
                self.selected = date;
                Task::none()
            }
            InMessage::InfiniteMsg(msg) => self
                .inf_view
                .update(msg, self.storage.as_ref())
                .map(Into::into),
            InMessage::DayMsg(msg) => self.day_view.update(msg).map(Into::into),
            InMessage::MonthMsg(msg) => self.month_view.update(msg).map(Into::into),
            InMessage::OpenStorage(storage) => {
                self.storage = Some(storage);
                Task::none()
            }
        }
    }
}
