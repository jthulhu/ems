use std::{iter::once, sync::Arc};

use chrono::{Duration, NaiveDate, NaiveTime, Timelike};
use icalendar::EventStatus;
use iced::{
    Background, Border, Color, Element, Length, Task,
    alignment::{Horizontal, Vertical},
    border::Radius,
    theme::palette::is_dark,
    widget::{
        Column, Id, Stack, column, container,
        operation::scroll_to,
        rich_text, row, rule, scrollable,
        scrollable::{AbsoluteOffset, Direction, Scrollbar},
        space, span, text,
    },
};
use moka::sync::Cache;

use crate::db::{Occurrence, TimeSpan};

struct AllDayOcc {
    title: Arc<str>,
    canceled: bool,
    color: Color,
}

struct TimedOcc {
    title: Arc<str>,
    start: NaiveTime,
    duration: Duration,
    canceled: bool,
    color: Color,
}

#[derive(Debug)]
pub struct DayView {
    scrollable_id: Id,
    cache: Cache<NaiveDate, Vec<Occurrence>>,
}

#[derive(Debug, Clone)]
pub enum InMessage {}

#[derive(Debug, Clone)]
pub enum UpMessage {}

pub type OutMessage = crate::ui::OutMessage<InMessage, UpMessage>;

impl From<InMessage> for OutMessage {
    fn from(msg: InMessage) -> Self {
        Self::InMessage(msg)
    }
}

impl From<UpMessage> for OutMessage {
    fn from(msg: UpMessage) -> Self {
        Self::UpMessage(msg)
    }
}

impl DayView {
    const HOUR_WIDTH: f32 = 40.0;

    const ALL_DAY_HEIGHT: f32 = 60.0;

    const HOUR_HEIGHT: f32 = 200.0;

    const DAY_START_TIME: NaiveTime = NaiveTime::from_hms_opt(8, 0, 0).unwrap();

    pub fn new(cache: Cache<NaiveDate, Vec<Occurrence>>) -> (Self, Task<!>) {
        let scrollable_id = Id::unique();
        (
            Self {
                scrollable_id: scrollable_id.clone(),
                cache,
            },
            scroll_to(
                scrollable_id,
                AbsoluteOffset {
                    x: None,
                    y: Some(
                        Self::DAY_START_TIME.num_seconds_from_midnight() as f32 / 3600.0
                            * Self::HOUR_HEIGHT,
                    ),
                },
            ),
        )
    }

    pub fn update(&mut self, msg: InMessage) -> Task<OutMessage> {
        match msg {}
    }

    pub fn view(&self, selected: NaiveDate) -> Element<'_, OutMessage> {
        let mut all_day_occ = Vec::new();
        let mut timed_occ = Vec::new();
        if let Some(occs) = self.cache.get(&selected) {
            let default_title: Arc<str> = Arc::from("(No Title)");
            for occ in occs {
                let canceled = matches!(occ.status, Some(EventStatus::Cancelled));

                let title = occ.summary.unwrap_or_else(|| default_title.clone());
                match occ.time_span {
                    TimeSpan::FullDay { .. } => all_day_occ.push(AllDayOcc {
                        title,
                        canceled,
                        color: occ.color,
                    }),
                    TimeSpan::DateTime { start, duration } => timed_occ.push(TimedOcc {
                        title,
                        start: start.time(),
                        duration,
                        canceled,
                        color: occ.color,
                    }),
                }
            }
        }
        column![
            row![
                space().width(Self::HOUR_WIDTH).height(Length::Shrink),
                Column::with_children(all_day_occ.into_iter().map(|occ| {
                    container(rich_text::<(), _, _, _>([
                        span(occ.title.to_string()).strikethrough(occ.canceled)
                    ]))
                    .style(move |_theme| container::Style {
                        text_color: if is_dark(occ.color) {
                            Some(Color::WHITE)
                        } else {
                            Some(Color::BLACK)
                        },
                        background: Some(Background::Color(occ.color)),
                        border: Border {
                            color: Color::BLACK,
                            width: 1.0,
                            radius: Radius::from(2.0),
                        },
                        ..Default::default()
                    })
                    .height(Self::ALL_DAY_HEIGHT)
                    .width(Length::Fill)
                    .into()
                }))
                .spacing(5.0)
            ],
            scrollable(row![
                Column::with_children((0..24).map(|h| {
                    container(container(text!("{h}:00").size(12.0)).padding(2.0))
                        .width(Length::Fill)
                        .height(Length::Fixed(Self::HOUR_HEIGHT))
                        .align_x(Horizontal::Right)
                        .align_y(Vertical::Top)
                        .into()
                }))
                .width(Self::HOUR_WIDTH),
                Stack::with_children(
                    once(
                        container(space())
                            .style(style::hour_box())
                            .width(Length::Fill)
                            .height(Length::Fixed(Self::HOUR_HEIGHT * 24.0))
                            .into()
                    )
                    .chain(timed_occ.into_iter().map(|occ| {
                        column![
                            space().height(
                                occ.start.num_seconds_from_midnight() as f32 / 3600.0
                                    * Self::HOUR_HEIGHT
                            ),
                            container(rich_text::<(), _, _, _>([
                                span(occ.title.to_string()).strikethrough(occ.canceled)
                            ]))
                            .width(Length::Fill)
                            .height(occ.duration.num_minutes() as f32 / 60.0 * Self::HOUR_HEIGHT)
                            .style(move |_theme| {
                                container::Style {
                                    text_color: if is_dark(occ.color) {
                                        Some(Color::WHITE)
                                    } else {
                                        Some(Color::BLACK)
                                    },
                                    background: Some(Background::Color(occ.color)),
                                    border: Border {
                                        color: Color::BLACK,
                                        width: 1.0,
                                        radius: Radius::from(2.0),
                                    },
                                    ..Default::default()
                                }
                            })
                        ]
                        .into()
                    }))
                    .chain(once(
                        Column::with_children((0..24).map(|_| {
                            column![
                                rule::horizontal(2).style(style::rule()),
                                space().height(Self::HOUR_HEIGHT - 2.0)
                            ]
                            .into()
                        }))
                        .into()
                    ))
                )
                .width(Length::Fill),
            ])
            .id(self.scrollable_id.clone())
            .direction(Direction::Vertical(Scrollbar::hidden()))
            .height(Length::Fill)
        ]
        .into()
    }
}

mod style {
    use iced::{
        Color, Theme,
        widget::{
            container,
            rule::{self, FillMode},
        },
    };

    pub fn hour_box() -> impl Fn(&Theme) -> container::Style {
        move |_theme| container::Style {
            background: Some(iced::Background::Color(Color::BLACK)),
            ..Default::default()
        }
    }

    pub fn rule() -> impl Fn(&Theme) -> rule::Style {
        move |theme| {
            let extended = theme.extended_palette();
            rule::Style {
                color: extended.background.strong.color,
                radius: 0.0.into(),
                fill_mode: FillMode::Full,
                snap: false,
            }
        }
    }
}
