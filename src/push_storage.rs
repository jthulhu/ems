use std::collections::HashMap;

use async_trait::async_trait;
use map_macro::hash_map;
use oo7::Keyring;
use unifiedpush::Distributor;
use unifiedpush_storage::{TokenInstance, UnifiedPushStorage};

pub struct PushStorage {
    app_id: String,
    keyring: Keyring,
}

impl PushStorage {
    async fn new(app_id: impl Into<String>) -> Result<Self, oo7::Error> {
        Ok(Self {
            app_id: app_id.into(),
            keyring: Keyring::new().await?,
        })
    }
}

trait SecretItem {
    fn attributes(app_id: &str) -> HashMap<&'static str, &'static str>;
    fn label() -> &'static str;
}

#[allow(unused_variables)]
#[async_trait]
impl UnifiedPushStorage for PushStorage {
    async fn distributor_get(&self) -> Option<Distributor> {
        if let Some(item) = self
            .keyring
            .search_items(&hash_map! {
                "application" => self.app_id.as_str(),
                "kind" => "unified-push/distributor",
            })
            .await
            .unwrap()
            .first()
        {
            toml::from_slice(&item.secret().await.unwrap()).ok()
        } else {
            None
        }
    }

    async fn distributor_set(&mut self, distributor: String) {
        todo!()
    }

    async fn distributor_ack(&mut self) {
        todo!()
    }

    async fn distributor_remove(&mut self) {
        todo!()
    }

    async fn key_get(&self, instance: &str) -> Option<String> {
        todo!()
    }

    async fn key_set(&mut self, instance: &str, key: &str) {
        todo!()
    }

    async fn key_remove(&mut self, instance: &str) {
        todo!()
    }

    async fn key_remove_all(&mut self) {
        todo!()
    }

    async fn registration_get_from_instance(&self, instance: &str) -> Option<TokenInstance> {
        todo!()
    }

    async fn registration_get_from_token(&self, token: &str) -> Option<TokenInstance> {
        todo!()
    }

    async fn registration_list(&self) -> Vec<TokenInstance> {
        todo!()
    }

    async fn registration_remove(&mut self, instance: &str) -> bool {
        todo!()
    }

    async fn registration_remove_all(&mut self) {
        todo!()
    }

    async fn registration_save(&mut self, token: &TokenInstance) {
        todo!()
    }
}
