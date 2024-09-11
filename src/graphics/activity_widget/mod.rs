// pub mod allocate_and_draw;
pub mod boxed_activity_mode;
pub mod imp;
pub mod layout_manager;
pub mod local_css_context;
mod object_subclass_impl;
use abi::{glib, gtk};
use gtk::prelude::*;

use self::boxed_activity_mode::ActivityMode;
use super::util;

glib::wrapper! {
    /// A Widget containing from 1 to 4 Widgets, one for each mode.
    /// It should contain at least the Minimal widget.
    ///
    /// It also stretches on drag if enabled
    ///
    /// The valign and halign properties of the mode widgets, along with the requested size,
    /// decide the size of the sub-widget during a mode change:
    /// view [`get_child_aligned_allocation`](super::util::get_child_aligned_allocation) for more info
    ///
    /// # Properties
    ///
    /// * `mode` (get,set) - The current mode of the ActivityWidget
    /// * `last-mode` (get) - The last mode of the ActivityWidget
    ///
    /// * `minimal-mode-widget` (get,set) - The widget to be shown in minimal mode
    /// * `compact-mode-widget` (get,set) - The widget to be shown in compact mode
    /// * `expanded-mode-widget` (get,set) - The widget to be shown in expanded mode
    /// * `overlay-mode-widget` (get,set) - The widget to be shown in overlay mode
    ///
    /// ## Properties used by dynisland
    /// These properties are automatically set from the general_style_config field in the config
    /// module implementers should only set them if the module allows the user to change them
    ///
    /// * `config-minimal-height` (get,set) - The minimum height of the ActivityWidget
    /// * `config-minimal-width` (get,set) - The minimum width of the ActivityWidget
    /// * `config-blur-radius` (get,set) - The blur radius of the ActivityWidget during a transition
    /// * `config-enable-drag-stretch` (get,set) - Whether the ActivityWidget can be stretched by dragging
    pub struct ActivityWidget(ObjectSubclass<imp::ActivityWidgetPriv>)
        @extends gtk::Widget;
}

impl Default for ActivityWidget {
    fn default() -> Self {
        let sel = glib::Object::new::<Self>();
        sel.set_overflow(gtk::Overflow::Hidden);

        sel
    }
}

impl ActivityWidget {
    pub fn new(name: &str) -> Self {
        let wid = Self::default();
        // wid.set_has_window(false);
        wid.set_name(name);
        wid
    }

    pub fn get_widget_for_mode(&self, mode: ActivityMode) -> Option<gtk::Widget> {
        match mode {
            ActivityMode::Minimal => self.minimal_mode_widget(),
            ActivityMode::Compact => self.compact_mode_widget(),
            ActivityMode::Expanded => self.expanded_mode_widget(),
            ActivityMode::Overlay => self.overlay_mode_widget(),
        }
    }

    pub fn current_widget(&self) -> Option<gtk::Widget> {
        self.get_widget_for_mode(self.mode())
    }
}
