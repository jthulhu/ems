use std::collections::{HashMap, hash_map::Entry};

use chrono::{DateTime, Duration, NaiveDate};
use chrono_tz::Tz;
use emer::raise;
use icalendar::{
    Calendar as ICalendar, CalendarDateTime, Component, DatePerhapsTime, Event, EventLike,
};
use iced::color;
use sea_orm::prelude::*;

use crate::{
    db::{Occurrence, TimeSpan},
    error::{ErrorKind, Result as EmsResult},
};

pub type ObjectEntity = Entity;
#[allow(non_upper_case_globals)]
pub const ObjectEntity: ObjectEntity = Entity;
#[allow(unused)]
pub type ObjectMetadata = Model;
pub type Object = ModelEx;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "calendar_object")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub href: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub calendar_href: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub account_user: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub account_server: String,
    pub etag: Option<String>,
    #[sea_orm(
        belongs_to,
        from = "(account_user, account_server, calendar_href)",
        to = "(account_user, account_server, href)"
    )]
    pub calendar: BelongsTo<super::calendar::Entity>,
    pub ical_data: String,
}

impl ActiveModelBehavior for ActiveModel {}

pub enum EndTime {
    Duration(Duration),
    TimeStamp(DateTime<Tz>),
}

pub enum EventTimes {
    WithTime {
        start: DateTime<Tz>,
        end: EndTime,
    },
    WholeDay {
        start: NaiveDate,
        /// Duration, in days, of the event.
        duration: u32,
    },
}

impl EventTimes {
    fn duration(&self) -> Duration {
        match self {
            EventTimes::WithTime {
                end: EndTime::Duration(duration),
                ..
            } => *duration,
            EventTimes::WithTime {
                start,
                end: EndTime::TimeStamp(end),
            } => *end - *start,
            EventTimes::WholeDay { duration, .. } => Duration::days(*duration as i64),
        }
    }

    fn start(&self) -> NaiveDate {
        match self {
            EventTimes::WithTime { start, .. } => start.date_naive(),
            EventTimes::WholeDay { start, .. } => *start,
        }
    }
}

fn with_timezone(dt: CalendarDateTime, tz: Tz) -> DateTime<Tz> {
    match dt {
        CalendarDateTime::Floating(dt) => dt.and_local_timezone(tz).single().unwrap(),
        CalendarDateTime::Utc(dt) => dt.with_timezone(&tz),
        CalendarDateTime::WithTimezone { date_time, tzid } => date_time
            .and_local_timezone(tzid.parse::<Tz>().unwrap())
            .single()
            .unwrap(),
    }
}

impl Object {
    /// Fetch all occurrences between `start` and `date`in this calendar object.
    pub fn occurrences(
        &self,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> EmsResult<HashMap<NaiveDate, Vec<Occurrence>>> {
        let tz = start.timezone();
        let start = start.with_timezone(&icalendar::Tz::Tz(start.timezone()));
        let end = end.with_timezone(&icalendar::Tz::Tz(end.timezone()));
        // Master events are event entries that generate (possible recurrent) occurrences.
        let mut main_events: HashMap<&str, &Event> = HashMap::new();
        // On the other hand, exception events are entries that either modify or delete an
        // occurrence of a repeating event.
        //
        // Note that this is not the only way to create exceptions: a single event itself might
        // declare both a recurring event, and some exceptions.  Those are handled transparently
        // by the library, so we don't need to care about them.
        let mut exception_events: HashMap<&str, Vec<&Event>> = HashMap::new();
        let object = self
            .ical_data
            .parse::<ICalendar>()
            .map_err(|error| raise!(ErrorKind::IcalParse(error)))?;

        let color = self
            .calendar
            .as_ref()
            .and_then(|cal| cal.color.as_ref()?.parse().ok())
            .unwrap_or_else(|| color!(0xFFAE42));

        for event in object.events() {
            let Some(uid) = event.get_uid() else {
                continue;
            };
            if event.get_recurrence_id().is_some() {
                exception_events.entry(uid).or_default().push(event);
            } else {
                match main_events.entry(uid) {
                    Entry::Occupied(mut entry)
                        if entry.get().get_sequence().unwrap_or(0)
                            < event.get_sequence().unwrap_or(0) =>
                    {
                        entry.insert(event);
                    }

                    Entry::Occupied(_) => {}
                    Entry::Vacant(entry) => drop(entry.insert(event)),
                }
            }
        }

        let mut result: HashMap<_, Vec<_>> = HashMap::new();
        for (uid, event) in main_events {
            let event_start = event.get_start().unwrap();
            let summary = event.get_summary().map(Into::into);
            let description = event.get_description().map(Into::into);
            let status = event.get_status();
            let exceptions = exception_events
                .remove(&uid)
                .unwrap_or_default()
                .into_iter()
                .map(|event| event.get_start().unwrap())
                .collect::<Vec<_>>();
            let time = match (
                event_start,
                event.get_end(),
                event.property_value("DURATION"),
            ) {
                // Forbidden: only one, among DURATION and DTEND, can be provided
                (_, Some(_), Some(_)) => panic!(),
                // If none of the above are specified, the duration is either:
                // - 1 day, if it's a full-day event
                // - 0s, otherwise
                (DatePerhapsTime::DateTime(start), None, None) => EventTimes::WithTime {
                    start: with_timezone(start, tz),
                    end: EndTime::Duration(Duration::zero()),
                },
                (DatePerhapsTime::Date(start), None, None) => {
                    EventTimes::WholeDay { start, duration: 1 }
                }
                (DatePerhapsTime::DateTime(start), None, Some(duration)) => {
                    let start = with_timezone(start, tz);
                    let duration = icalendar_duration::parse(duration).unwrap();
                    let end = start + duration;
                    debug_assert!(start <= end);
                    let duration = end - start;
                    EventTimes::WithTime {
                        start,
                        end: EndTime::Duration(duration),
                    }
                }
                (DatePerhapsTime::Date(start), Some(DatePerhapsTime::Date(end)), None) => {
                    EventTimes::WholeDay {
                        start,
                        duration: (end - start).num_days() as _,
                    }
                }
                (DatePerhapsTime::Date(_), Some(DatePerhapsTime::DateTime(_)), None) => panic!(),
                (DatePerhapsTime::Date(start), None, Some(duration)) => {
                    let duration = icalendar_duration::parse(duration).unwrap();
                    let end = (start.and_hms_opt(0, 0, 0).unwrap() + duration).date();
                    debug_assert!(start <= end);
                    EventTimes::WholeDay {
                        start,
                        duration: (end - start).num_days() as _,
                    }
                }
                (DatePerhapsTime::DateTime(start), Some(DatePerhapsTime::DateTime(end)), None) => {
                    EventTimes::WithTime {
                        start: with_timezone(start, tz),
                        end: EndTime::TimeStamp(with_timezone(end, tz)),
                    }
                }
                (DatePerhapsTime::DateTime(_), Some(DatePerhapsTime::Date(_)), None) => panic!(),
            };
            let occs = event
                .get_recurrence()
                .map_err(|error| raise!(ErrorKind::IcalRecurrence(error)))?;
            for occ in occs.after(start).before(end).all(400).dates {
                if exceptions.iter().any(|date| match date {
                    DatePerhapsTime::DateTime(date) => occ == with_timezone(date.clone(), tz),
                    DatePerhapsTime::Date(date) => occ.date_naive() == *date,
                }) {
                    continue;
                }
                result.entry(time.start()).or_default().push(Occurrence {
                    description: description.clone(),
                    summary: summary.clone(),
                    color,
                    status,
                    time_span: match time {
                        EventTimes::WithTime {
                            start,
                            end: EndTime::Duration(duration),
                        } => TimeSpan::DateTime { start, duration },
                        EventTimes::WithTime {
                            start,
                            end: EndTime::TimeStamp(end),
                        } => TimeSpan::DateTime {
                            start,
                            duration: end - start,
                        },
                        EventTimes::WholeDay { start, duration } => TimeSpan::FullDay {
                            day: start,
                            duration,
                        },
                    },
                });
            }
        }
        Ok(result)
    }
}
