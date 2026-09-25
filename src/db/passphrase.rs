use std::{array, mem::take};

use emer::raise;
use map_macro::hash_map;
use oo7::{Keyring, Secret};

use crate::error::{ErrorKind, Result};

pub type Key = [u8; 128];

pub async fn get_passphrase(app_name: &str, database: &str) -> Result<Box<Key>> {
    let attributes = hash_map! {
        "application" => app_name,
        "kind" => "database-key",
        "database" => database,
    };

    let keyring = Keyring::new()
        .await
        .map_err(|error| raise!(ErrorKind::Keyring(error)))?;
    let items = keyring
        .search_items(&attributes)
        .await
        .map_err(|error| raise!(ErrorKind::Keyring(error)))?;
    if let Some(item) = items.first() {
        let secret = match item
            .secret()
            .await
            .map_err(|error| raise!(ErrorKind::Keyring(error)))?
        {
            Secret::Text(ref mut text) => take(text).into_bytes(),
            Secret::Blob(ref mut bytes) => take(bytes),
        };
        if let Some(key) = secret.into_boxed_slice().try_into().ok() {
            return Ok(key);
        }
    }
    let secret: Box<Key> = Box::new(array::from_fn(|_| rand::random()));
    keyring
        .create_item("MS database key", &attributes, &*secret, true)
        .await
        .map_err(|error| raise!(ErrorKind::Keyring(error)))?;
    Ok(secret)
}
