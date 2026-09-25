use chrono::{Datelike, Duration, NaiveDate, Weekday};
use iced::{
    Element, Length, Task,
    widget::{Row, button, column, container, text},
};

#[derive(Debug, Clone)]
pub enum UpMessage {
    Select(NaiveDate),
}

#[derive(Debug, Clone)]
pub enum InMessage {}

pub type OutMessage = crate::ui::OutMessage<InMessage, UpMessage>;

impl From<UpMessage> for OutMessage {
    fn from(msg: UpMessage) -> Self {
        Self::UpMessage(msg)
    }
}

impl From<InMessage> for OutMessage {
    fn from(msg: InMessage) -> Self {
        Self::InMessage(msg)
    }
}

#[derive(Debug)]
pub struct MonthView {}

impl MonthView {
    pub fn new() -> (Self, Task<!>) {
        (Self {}, Task::none())
    }

    pub fn update(&mut self, message: InMessage) -> Task<OutMessage> {
        match message {}
    }

    pub fn view(&self, selected: NaiveDate) -> Element<'_, OutMessage> {
        let mut current_day =
            NaiveDate::from_ymd_opt(selected.year(), selected.month(), 1).unwrap();
        while current_day.weekday() != Weekday::Mon {
            current_day -= Duration::days(1);
        }
        let mut month = column![].spacing(5);
        while (current_day.month() <= selected.month() && current_day.year() <= selected.year())
            || current_day.year() < selected.year()
            || current_day.weekday() != Weekday::Mon
        {
            month = month.push(Row::with_children((0..7).map(|_| {
                let day = button(text(current_day.day()).center().width(30.0))
                    .style(style::day(if current_day == selected {
                        style::DayKind::Selected
                    } else if current_day.month() == selected.month() {
                        style::DayKind::CurrentMonth
                    } else {
                        style::DayKind::Other
                    }))
                    .on_press(UpMessage::Select(current_day).into());
                current_day += Duration::days(1);
                day.into()
            })));
        }
        container(
            container(month)
                .style(container::rounded_box)
                .height(Length::Shrink)
                .width(Length::Shrink)
                .padding(5),
        )
        .center_x(Length::Fill)
        .height(Length::Shrink)
        .into()
    }
}

mod style {
    use iced::{
        Background, Border, Theme,
        widget::button::{self, Status},
    };

    #[derive(Debug, Clone, Copy)]
    pub enum DayKind {
        Selected,
        CurrentMonth,
        Other,
    }

    pub fn day(kind: DayKind) -> impl Fn(&Theme, Status) -> button::Style {
        move |theme: &Theme, status: Status| {
            let extended = theme.extended_palette();
            let (fg, bg, border) = match status {
                Status::Active => (
                    match kind {
                        DayKind::Selected => extended.primary.strong,
                        DayKind::CurrentMonth => extended.primary.base,
                        DayKind::Other => extended.primary.weak,
                    }
                    .text,
                    extended.background.neutral.color,
                    None,
                ),
                Status::Hovered => (
                    match kind {
                        DayKind::Selected => extended.primary.strong,
                        DayKind::CurrentMonth => extended.primary.base,
                        DayKind::Other => extended.primary.weak,
                    }
                    .text,
                    extended.background.neutral.color,
                    Some(extended.primary.strong.color),
                ),
                Status::Pressed => (
                    extended.success.base.text,
                    extended.background.neutral.color,
                    Some(extended.success.base.color),
                ),
                Status::Disabled => (
                    extended.secondary.base.text,
                    extended.background.neutral.color,
                    None,
                ),
            };
            button::Style {
                text_color: fg,
                background: Some(Background::Color(bg)),
                border: if let Some(border) = border {
                    Border {
                        color: border,
                        width: 1.0,
                        radius: 2.0.into(),
                    }
                } else {
                    Border::default()
                },
                ..Default::default()
            }
        }
    }
}
