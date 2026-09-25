use std::sync::Arc;

use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone, Weekday};
use chrono_tz::Tz;
use derive_more::From;
use iced::{
    Background, Color, Element, Font, Length, Task, alignment,
    font::Weight,
    never,
    theme::palette::is_dark,
    widget::{
        Column, Id, Row, button, column, container,
        operation::{AbsoluteOffset, scroll_to},
        rich_text, row, scrollable,
        scrollable::{Direction, Scrollbar, Viewport},
        space, span, text,
    },
};
use moka::sync::Cache;

use crate::{
    db::{Occurrence, Storage, TimeSpan},
    error::{Error, Result},
};

#[derive(Debug, Clone, From)]
pub enum InMessage {
    #[from(skip)]
    Scrolled(Viewport),
    #[from(())]
    FinishedCaching,
}

#[derive(Debug, Clone)]
pub enum UpMessage {
    Select(NaiveDate),
    Error(Arc<Error>),
}

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

impl From<Result<()>> for OutMessage {
    fn from(err: Result<()>) -> Self {
        match err {
            Ok(()) => OutMessage::InMessage(InMessage::FinishedCaching),
            Err(error) => OutMessage::UpMessage(UpMessage::Error(Arc::new(error))),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Boundary {
    None,
    Month,
    Year,
}

impl From<NaiveDate> for Boundary {
    fn from(date: NaiveDate) -> Self {
        let yesterday = date - Duration::days(1);
        let tomorrow = date + Duration::days(1);
        if yesterday.year() != date.year() || tomorrow.year() != date.year() {
            Boundary::Year
        } else if yesterday.month() != date.month() || tomorrow.month() != date.month() {
            Boundary::Month
        } else {
            Boundary::None
        }
    }
}

#[derive(Debug)]
pub struct InfiniteView {
    cache: Cache<NaiveDate, Vec<Occurrence>>,
    pivot_date: NaiveDate,
    scroll_offset: f32,
    view_size: f32,
    scrollable_id: Id,
    first: bool,
}

impl InfiniteView {
    const EVENT_HEIGHT: f32 = Self::DAY_HEIGHT / 5.0;
    const DAY_HEIGHT: f32 = 150.0;
    const BUFFER_HEIGHT: f32 = Self::DAY_HEIGHT * 100.0;
    const CENTER: f32 = Self::BUFFER_HEIGHT / 2.0;
    const RESET_THRESHOLD: f32 = Self::DAY_HEIGHT * 5.0;

    fn scroll_start_date(&self) -> (f32, NaiveDate) {
        let pivot_date_offset = Self::CENTER - self.view_size / 2.0;
        let weeks_offset = ((self.scroll_offset - pivot_date_offset) / Self::DAY_HEIGHT).floor();
        let start_offset = pivot_date_offset + weeks_offset * Self::DAY_HEIGHT;
        // We start drawing *before* the beginning of the viewport
        debug_assert!(start_offset <= self.scroll_offset);
        let mut snap_date = self.pivot_date + Duration::weeks(weeks_offset as i64);
        while snap_date.weekday() != Weekday::Mon {
            snap_date = snap_date.pred_opt().unwrap();
        }
        (start_offset, snap_date)
    }

    pub fn new(cache: Cache<NaiveDate, Vec<Occurrence>>) -> (Self, Task<!>) {
        let scrollable_id = Id::unique();
        let today = Local::now().date_naive();
        (
            Self {
                cache,
                pivot_date: today,
                scroll_offset: 0.0,
                view_size: 0.0,
                scrollable_id: scrollable_id.clone(),
                first: true,
            },
            scroll_to(
                scrollable_id,
                AbsoluteOffset {
                    x: None,
                    y: Some(Self::CENTER),
                },
            ),
        )
    }

    pub fn update(&mut self, message: InMessage, storage: Option<&Storage>) -> Task<OutMessage> {
        match message {
            InMessage::Scrolled(viewport) => {
                if self.first {
                    self.first = false;
                    return Task::none();
                }
                self.view_size = viewport.bounds().height;
                self.scroll_offset = viewport.absolute_offset().y;
                if self.scroll_offset < Self::RESET_THRESHOLD
                    || self.scroll_offset > Self::BUFFER_HEIGHT - Self::RESET_THRESHOLD
                {
                    let delta = self.scroll_offset - Self::CENTER;
                    let weeks_delta = (delta / Self::DAY_HEIGHT).round();
                    let actual_delta = weeks_delta * Self::DAY_HEIGHT;
                    self.scroll_offset -= actual_delta;
                    self.pivot_date += Duration::weeks(weeks_delta as i64);
                    scroll_to(
                        self.scrollable_id.clone(),
                        AbsoluteOffset {
                            x: None,
                            y: Some(self.scroll_offset),
                        },
                    )
                } else if let Some(db) = storage.cloned() {
                    let (mut offset, start_date) = self.scroll_start_date();
                    let mut end_date = start_date;
                    let mut is_in_cache = true;
                    while offset <= self.scroll_offset + self.view_size {
                        for _ in 0..7 {
                            is_in_cache &= self.cache.contains_key(&end_date);
                            end_date += Duration::days(1);
                        }
                        offset += Self::DAY_HEIGHT;
                    }
                    if !is_in_cache {
                        let cache = self.cache.clone();
                        Task::perform(
                            async move {
                                let tz = iana_time_zone::get_timezone()
                                    .unwrap()
                                    .parse::<Tz>()
                                    .unwrap();
                                let cals = db.calendars().await?;
                                for cal in cals {
                                    for object in cal.objects {
                                        let occurrences = object.occurrences(
                                            tz.from_local_datetime(
                                                &(start_date - Duration::weeks(8))
                                                    .and_hms_opt(0, 0, 0)
                                                    .unwrap(),
                                            )
                                            .unwrap(),
                                            tz.from_local_datetime(
                                                &(end_date + Duration::weeks(8))
                                                    .and_hms_opt(0, 0, 0)
                                                    .unwrap(),
                                            )
                                            .unwrap(),
                                        )?;
                                        for (day, occurrences) in occurrences {
                                            cache.insert(day, occurrences);
                                        }
                                    }
                                }
                                Ok(())
                            },
                            Into::into,
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            InMessage::FinishedCaching => Task::none(),
        }
    }

    pub fn view(&self, selected: NaiveDate) -> Element<'_, OutMessage> {
        let (mut start_offset, mut curr_day) = self.scroll_start_date();
        let mut scroll_view = column![
            container(space().width(Length::Fill).height(start_offset)).style(|_| {
                container::Style {
                    background: Some(Background::Color(Color::from_rgb(1.0, 0.0, 0.0))),
                    ..Default::default()
                }
            }),
        ]
        .spacing(0);
        while start_offset <= self.scroll_offset + self.view_size {
            let week = Row::with_children((0..7).map(|_| {
                let res = button(
                    container(column![
                        container(
                            container(
                                text(
                                    curr_day
                                        .format(match curr_day.into() {
                                            Boundary::Year => "%Y %b %d",
                                            Boundary::Month => "%b %d",
                                            Boundary::None => "%d",
                                        })
                                        .to_string()
                                )
                                .style(style::label(curr_day.month0()))
                            )
                            .padding(4)
                            .height(Length::Shrink)
                            .width(Length::Shrink)
                        )
                        .height(Length::Fixed(Self::EVENT_HEIGHT))
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Right),
                        scrollable(
                            Column::with_children(self.cache.get(&curr_day).into_iter().flat_map(
                                |occs| occs.into_iter().map(|occ| {
                                    let color = if is_dark(occ.color) {
                                        Color::WHITE
                                    } else {
                                        Color::BLACK
                                    };
                                    container(
                                        rich_text![
                                            if let TimeSpan::DateTime { start, .. } = occ.time_span
                                            {
                                                span(format!("{} ", start.time()))
                                                    .font(Font {
                                                        weight: Weight::Bold,
                                                        ..Default::default()
                                                    })
                                                    .color(color)
                                            } else {
                                                span("")
                                            },
                                            span(
                                                occ.summary
                                                    .as_deref()
                                                    .unwrap_or("(No Title)")
                                                    .to_string()
                                            )
                                            .color(color)
                                        ]
                                        .on_link_click(never),
                                    )
                                    .style(style::event(occ.color))
                                    .height(50.)
                                    .padding(5.0)
                                    .width(Length::Fill)
                                    .into()
                                })
                            ))
                            .spacing(5)
                        ),
                    ])
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(Self::DAY_HEIGHT))
                    .style(style::day(curr_day.month0() as u8, curr_day == selected)),
                )
                .on_press(UpMessage::Select(curr_day).into())
                .padding(0);
                curr_day += Duration::days(1);
                res.into()
            }))
            .spacing(0)
            .height(Length::Fixed(Self::DAY_HEIGHT))
            .width(Length::Fill);

            start_offset += Self::DAY_HEIGHT;
            scroll_view = scroll_view.push(week);
        }
        let scroll_view = scroll_view.push(
            space()
                .width(Length::Fill)
                .height(Self::BUFFER_HEIGHT - start_offset),
        );
        column![
            row![
                text("Monday")
                    .width(Length::FillPortion(1))
                    .align_x(text::Alignment::Center),
                text("Tuesday")
                    .width(Length::FillPortion(1))
                    .align_x(text::Alignment::Center),
                text("Wednesday")
                    .width(Length::FillPortion(1))
                    .align_x(text::Alignment::Center),
                text("Thursday")
                    .width(Length::FillPortion(1))
                    .align_x(text::Alignment::Center),
                text("Friday")
                    .width(Length::FillPortion(1))
                    .align_x(text::Alignment::Center),
                text("Saturday")
                    .width(Length::FillPortion(1))
                    .align_x(text::Alignment::Center),
                text("Sunday")
                    .width(Length::FillPortion(1))
                    .align_x(text::Alignment::Center),
            ]
            .width(Length::Fill)
            .height(Length::Shrink)
            .spacing(10),
            scrollable(scroll_view)
                .id(self.scrollable_id.clone())
                .direction(Direction::Vertical(Scrollbar::hidden()))
                .on_scroll(|viewport| InMessage::Scrolled(viewport).into())
                .height(Length::Fill)
                .width(Length::Fill)
        ]
        .height(Length::Fill)
        .width(Length::FillPortion(3))
        .into()
    }
}

mod style {
    use iced::{
        Background, Border, Color, Theme,
        border::Radius,
        color,
        widget::{container, text},
    };

    const MONTH_BG: [Color; 12] = [
        color!(0x4A6984),
        color!(0x6B526E),
        color!(0x556B5C),
        color!(0xB57C8A),
        color!(0x6E8C75),
        color!(0xCBB07E),
        color!(0xCD7F6D),
        color!(0x7C7393),
        color!(0xB88A5B),
        color!(0xA6634B),
        color!(0x4E5A44),
        color!(0x2F484B),
    ];

    const MONTH_FG: [Color; 12] = [
        color!(0xFFFFFF),
        color!(0xFFFFFF),
        color!(0xFFFFFF),
        color!(0x1E1E1E),
        color!(0xFFFFFF),
        color!(0x1E1E1E),
        color!(0x1E1E1E),
        color!(0xFFFFFF),
        color!(0x1E1E1E),
        color!(0xFFFFFF),
        color!(0xFFFFFF),
        color!(0xFFFFFF),
    ];

    const MONTH_HIGHLIGHT: [Color; 12] = [
        color!(0x6A8FAF),
        color!(0x8F7194),
        color!(0x74937E),
        color!(0xD69AA9),
        color!(0x8CB295),
        color!(0xE8CD9B),
        color!(0xEB9C8A),
        color!(0x9B91B5),
        color!(0xD9A979),
        color!(0xC77F65),
        color!(0x6C7D5F),
        color!(0x45686C),
    ];

    #[inline]
    pub fn day(month: u8, selected: bool) -> impl Fn(&Theme) -> container::Style {
        move |theme: &Theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(Background::Color(if selected {
                    let res = MONTH_HIGHLIGHT[month as usize];
                    res
                } else {
                    MONTH_BG[month as usize]
                })),
                border: Border {
                    color: palette.background.weak.color,
                    width: 2.0,
                    radius: Radius::new(0),
                },
                ..Default::default()
            }
        }
    }

    #[inline]
    pub fn event(color: Color) -> impl Fn(&Theme) -> container::Style {
        move |_theme: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                color,
                width: 0.0,
                radius: Radius::from(2.0),
            },
            ..Default::default()
        }
    }

    #[inline]
    pub fn label(month: u32) -> impl Fn(&Theme) -> text::Style {
        move |_theme| text::Style {
            color: Some(MONTH_FG[month as usize]),
        }
    }
}
