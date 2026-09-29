use std::{
    collections::HashSet,
    io,
    iter::once,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use emer::{raise, throw};
use iced::{
    Color, Element, Font, Pixels,
    font::{Style, Weight},
    widget::{
        Column, span,
        text::{Rich, Span},
    },
};
use libpass::StoreEntry;
use migrations::TutaMigrationHandler;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseConnection,
    EntityTrait, QueryFilter, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

use crate::{
    error::{ErrorKind, Result},
    uring::{self, DiskInterface},
};

use super::passphrase::get_passphrase;

mod asset;
mod jmap_account;
mod jmap_email;
mod jmap_email_attachment;
mod jmap_email_content;
mod jmap_email_has_address;
mod jmap_email_has_asset;
mod jmap_email_has_keyword_in_folder;
mod jmap_email_in_folder;
mod jmap_folder;

mod migrations;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailBody {
    pub body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub children: Vec<FlowElement>,
}

impl<'a, Msg: 'a> From<Body> for Element<'a, Msg> {
    fn from(value: Body) -> Self {
        value
            .children
            .into_iter()
            .map(Into::into)
            .collect::<Column<'a, Msg>>()
            .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowElement {
    Div {
        children: Vec<FlowElement>,
    },
    Table {
        rows: Vec<TableRow>,
    },
    Paragraph {
        children: Vec<PhraseElement>,
    },
    Heading {
        level: u8,
        children: Vec<PhraseElement>,
    },
}

impl<'a, Msg: 'a> From<FlowElement> for Element<'a, Msg> {
    fn from(value: FlowElement) -> Self {
        match value {
            FlowElement::Div { children } => children
                .into_iter()
                .map(Into::into)
                .collect::<Column<'a, Msg>>()
                .into(),
            FlowElement::Table { rows } => todo!(),
            FlowElement::Paragraph { children } => children
                .into_iter()
                .flat_map(PhraseElement::spans)
                .collect::<Rich<'a, String, Msg>>()
                .into(),
            FlowElement::Heading { level, children } => todo!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub children: Vec<FlowElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhraseElement {
    Text(String),
    Span {
        children: Vec<PhraseElement>,
    },
    Strong {
        children: Vec<PhraseElement>,
    },
    Emphasis {
        children: Vec<PhraseElement>,
    },
    Link {
        href: String,
        children: Vec<PhraseElement>,
    },
    Image {
        src: String,
        description: String,
        width: u32,
        height: u32,
    },
}

#[derive(Debug, Clone)]
struct SpanStyle {
    color: Option<Color>,
    size: Option<Pixels>,
    link: Option<String>,
    underline: bool,
    bold: bool,
    strikethrough: bool,
    emph: bool,
    mono: bool,
}

impl Default for SpanStyle {
    fn default() -> Self {
        Self {
            color: None,
            size: None,
            link: None,
            underline: false,
            bold: false,
            strikethrough: false,
            emph: false,
            mono: false,
        }
    }
}

impl PhraseElement {
    fn spans<'a>(self) -> Vec<Span<'a, String>> {
        self.spans_with(Default::default()).collect()
    }

    fn spans_with(self, style: SpanStyle) -> Box<dyn Iterator<Item = Span<'static, String>>> {
        match self {
            PhraseElement::Text(string) => {
                let mut span = span(string)
                    .color_maybe(style.color)
                    .underline(style.underline)
                    .link_maybe(style.link)
                    .font_maybe(if style.emph {
                        Some(Font {
                            style: Style::Italic,
                            ..Default::default()
                        })
                    } else {
                        None
                    })
                    .font_maybe(if style.bold {
                        Some(Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        })
                    } else {
                        None
                    })
                    .font_maybe(if style.mono {
                        Some(Font::MONOSPACE)
                    } else {
                        None
                    })
                    .strikethrough(style.strikethrough);
                if let Some(size) = style.size {
                    span = span.size(size);
                }
                Box::new(once(span))
            }
            PhraseElement::Span { children } => Box::new(
                children
                    .into_iter()
                    .flat_map(move |child| child.spans_with(style.clone())),
            ),
            PhraseElement::Strong { children } => {
                Box::new(children.into_iter().flat_map(move |child| {
                    child.spans_with(SpanStyle {
                        bold: true,
                        ..style.clone()
                    })
                }))
            }
            PhraseElement::Emphasis { children } => {
                Box::new(children.into_iter().flat_map(move |child| {
                    child.spans_with(SpanStyle {
                        emph: true,
                        ..style.clone()
                    })
                }))
            }
            PhraseElement::Link { href, children } => {
                Box::new(children.into_iter().flat_map(move |child| {
                    child.spans_with(SpanStyle {
                        link: Some(href.clone()),
                        ..style.clone()
                    })
                }))
            }
            PhraseElement::Image {
                src,
                description,
                width,
                height,
            } => todo!(),
        }
    }
}

/// File mapper for the Tuta SDK.  Tuta abstracts over the filesystem, asking for a key/value
/// map.  Storing the values in a database is a bad idea, Tuta is already implementing an
/// encrypted database.
pub struct FileMapper {
    email: String,
    base_dir: BaseDirectories,
    disk: DiskInterface,
}

impl FileMapper {
    pub fn new(email: impl ToString, base_dir: BaseDirectories, disk: DiskInterface) -> Self {
        Self {
            email: email.to_string(),
            base_dir,
            disk,
        }
    }

    fn file_to_store(&self, key: String) -> std::result::Result<PathBuf, io::Error> {
        self.base_dir
            .place_data_file(format!("{}/{key}", self.email))
    }
}

#[derive(Debug, Clone)]
pub struct EmailStorage {
    email_db: DatabaseConnection,
}

pub struct TutaAccount {}

impl EmailStorage {
    pub async fn open(email_path: impl AsRef<Path>, app_name: &str) -> Result<Self> {
        let email_db = {
            let email_passphrase = get_passphrase(app_name, "email").await?;
            let mut options = ConnectOptions::new(format!(
                "sqlite://{}?mode=rwc",
                email_path.as_ref().to_str().unwrap()
            ));
            options.set_application_name(app_name);
            options.sqlcipher_key(hex::encode(&*email_passphrase));
            options.max_connections(1);
            let email_db = Database::connect(options)
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            email_db
                .execute_unprepared(
                    r#"
                        pragma journal_mode = WAL;
                        pragma synchronous = NORMAL;
                    "#,
                )
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            email_db
        };

        Ok(Self { email_db })
    }
}
