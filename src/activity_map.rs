use std::{collections::HashMap, rc::Rc, sync::Arc};

use abi::module::ActivityIdentifier;
use anyhow::{anyhow, bail, Result};
use tokio::sync::Mutex;

use crate::{dynamic_activity::DynamicActivity, dynamic_property::DynamicPropertyAny};

#[derive(Default)]
pub struct ActivityMap {
    pub(super) map: HashMap<String, Rc<Mutex<DynamicActivity>>>,
}

impl ActivityMap {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_activity(&self, identifier: &str) -> Result<Rc<Mutex<DynamicActivity>>> {
        self.map
            .get(identifier)
            .cloned()
            .ok_or_else(|| anyhow!("Activity {} not found", identifier))
    }
    pub fn insert_activity(&mut self, activity: Rc<Mutex<DynamicActivity>>) -> Result<()> {
        let activity_id = activity.blocking_lock().get_identifier();
        if self.map.contains_key(activity_id.activity()) {
            bail!("activity {} was already registered", activity_id);
        }
        self.map
            .insert(activity_id.activity().to_string(), activity);
        Ok(())
    }
    pub fn remove_activity(&mut self, activity_id: &ActivityIdentifier) -> Result<()> {
        if !self.map.contains_key(activity_id.activity()) {
            bail!("activity {} wasn't registered", activity_id);
        }
        self.map.remove(activity_id.activity());
        Ok(())
    }
    pub fn list_activity_names(&self) -> Vec<&str> {
        self.map.keys().map(|x| x.as_str()).collect()
    }
    pub fn list_activity_windows(&self) -> Vec<Option<String>> {
        self.map
            .values()
            .map(|x| x.blocking_lock().identifier.metadata().window_name())
            .collect()
    }
    pub fn list_activities(&self) -> Vec<ActivityIdentifier> {
        self.map
            .values()
            .map(|x| x.blocking_lock().get_identifier())
            .collect()
    }
    /// Get a property from an activity
    ///
    /// blocking
    pub fn get_property_any_blocking(
        &self,
        activity_name: &str,
        property_name: &str,
    ) -> Result<Arc<Mutex<DynamicPropertyAny>>> {
        self.get_activity(activity_name)?
            .blocking_lock()
            .get_property_any(property_name)
    }
    /// Get a property from an activity
    pub async fn get_property_any(
        &self,
        activity_name: &str,
        property_name: &str,
    ) -> Result<Arc<Mutex<DynamicPropertyAny>>> {
        self.get_activity(activity_name)?
            .lock()
            .await
            .get_property_any(property_name)
    }
}
