use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, bail, Ok, Result};
use dyn_clone::DynClone;
use dynisland_abi::{gtk, module::ActivityIdentifier};
use gtk::prelude::WidgetExt;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use super::graphics::activity_widget::ActivityWidget;
use crate::dynamic_property::{DynamicPropertyAny, PropertyUpdate, ValidDynType};

/// A closure that takes a `ValidDynType` and is cloneable
pub trait ValidDynamicClosure: Fn(&dyn ValidDynType) + DynClone {}
impl<T: Fn(&dyn ValidDynType) + DynClone + Clone> ValidDynamicClosure for T {}

impl Clone for Box<dyn ValidDynamicClosure> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(self.as_ref())
    }
}

/// Bundles a `DynamicProperty` with all of its subscribers
pub struct SubscribableProperty {
    pub(crate) property: Arc<Mutex<DynamicPropertyAny>>,
    pub(crate) subscribers: Vec<Box<dyn ValidDynamicClosure>>,
}

/// Struct containing the `ActivityWidget`, the `ActivityIdentifier` and the dynamic properties of an activity
pub struct DynamicActivity {
    pub(crate) widget: ActivityWidget,
    pub(crate) property_dictionary: HashMap<String, SubscribableProperty>,
    pub(crate) prop_send: UnboundedSender<PropertyUpdate>,
    pub(crate) identifier: ActivityIdentifier,
}

impl DynamicActivity {
    /// Create a new DynamicActivity
    ///
    /// Also creates a new ActivityWidget
    ///
    /// * `prop_send` - the backend channel for the property update notifications, you get this from `BaseModule.prop_send()`
    pub fn new(
        prop_send: UnboundedSender<PropertyUpdate>,
        module_name: &str,
        activity_name: &str,
    ) -> Self {
        Self {
            widget: ActivityWidget::new(&(activity_name.to_string() + "-" + module_name)),
            property_dictionary: HashMap::new(),
            prop_send,
            identifier: ActivityIdentifier::new(module_name, activity_name),
        }
    }

    /// Create a new DynamicActivity with additional metadata
    ///
    /// The final activity name used in the identifier will be `"activity_name-window_name"`
    /// or just `"activity_name"` if `window_name` is `None`
    ///
    /// Also creates a new ActivityWidget
    ///
    /// # Arguments
    /// * `prop_send` - the backend channel for the property update notifications, you get this from `BaseModule.prop_send()`
    /// * `module_name` - the name of the module, use the one defined in the crate (example: `crate::NAME`)
    /// * `activity_name` - the base name of the activity, this should be unique for each window
    /// * `window_name` - the name of the window, this can be `None` and it will go the default window
    /// * `additional_metadata` - a list of additional metadata key-value pairs to add to the activity identifier
    pub fn new_with_metadata(
        prop_send: UnboundedSender<PropertyUpdate>,
        module_name: &str,
        activity_name: &str,
        window_name: Option<&str>,
        additional_metadata: Vec<(String, String)>,
    ) -> Self {
        let name = if let Some(window_name) = window_name {
            format!("{}-{}", activity_name, window_name)
        } else {
            format!("{}", activity_name)
        };
        let mut id = ActivityIdentifier::new(module_name, &name);
        if let Some(window_name) = window_name {
            id.metadata_mut().set_window_name(window_name);
        }
        let meta_map = id.metadata_mut();
        for (key, value) in additional_metadata {
            meta_map.set_additional_metadata(key, value);
        }
        let widget = ActivityWidget::new(&&(name.to_string() + "-" + module_name));
        widget.add_css_class(activity_name);

        Self {
            widget: widget,
            property_dictionary: HashMap::new(),
            prop_send,
            identifier: id,
        }
    }

    /// Replace the `ActivityWidget`
    ///
    /// # Warning
    /// This doesn't deallocate the previous ActivityWidget if the dynamic activity is registered,
    /// you have to do it before calling this method
    pub fn set_activity_widget(&mut self, widget: ActivityWidget) {
        widget.set_name(self.widget.name());
        self.widget = widget;
    }
    pub fn get_activity_widget(&self) -> ActivityWidget {
        self.widget.clone()
    }
    pub fn get_identifier(&self) -> ActivityIdentifier {
        self.identifier.clone()
    }

    /// Adds a dynamic property to itself
    ///
    /// The property has the type T and it can't be changed.
    ///
    /// Returns `Err` if the property already exists
    pub fn add_dynamic_property<T>(&mut self, name: &str, initial_value: T) -> Result<()>
    where
        T: ValidDynType,
    {
        if self.property_dictionary.contains_key(name) {
            bail!("propery already added")
        }
        let prop = DynamicPropertyAny {
            backend_channel: self.prop_send.clone(),
            activity_id: self.get_identifier(),
            property_name: name.to_string(),
            value: Box::new(initial_value),
        };
        let subs_prop = SubscribableProperty {
            property: Arc::new(Mutex::new(prop)),
            subscribers: Vec::new(),
        };
        self.property_dictionary.insert(name.to_string(), subs_prop);
        Ok(())
    }

    /// Adds a subscriber for when the property changes
    ///
    /// Returns `Err` if the property doesn't exist
    pub fn subscribe_to_property<F>(&mut self, name: &str, callback: F) -> Result<()>
    where
        F: ValidDynamicClosure + 'static,
    {
        let prop = self
            .property_dictionary
            .get_mut(name)
            .ok_or_else(|| anyhow!("property {} doesn't exist on this activity", name))?;
        prop.subscribers.push(Box::new(callback));
        Ok(())
    }

    /// Get all of the subscribers for a property
    pub fn get_subscribers(&self, name: &str) -> Result<&[Box<dyn ValidDynamicClosure>]> {
        let prop = self
            .property_dictionary
            .get(name)
            .ok_or_else(|| anyhow!("property {} doesn't exist on this activity", name))?;
        Ok(prop.subscribers.as_slice())
    }
    /// Get a reference to a dynamic property to get or change its value
    ///
    /// returns `Err` if the property doesn't exist
    pub fn get_property_any(&self, name: &str) -> Result<Arc<Mutex<DynamicPropertyAny>>> {
        match self.property_dictionary.get(name) {
            Some(property) => Ok(property.property.clone()),
            None => bail!("property {} doesn't exist on this activity", name),
        }
    }
}
