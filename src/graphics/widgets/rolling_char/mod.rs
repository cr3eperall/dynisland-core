pub mod imp;
mod object_subclass_impl;

use abi::{glib, gtk};
use gtk::{prelude::*, subclass::prelude::*};

glib::wrapper! {
    /// A Label containing a single char that scrolls up when it is changed
    ///
    /// # Properties
    /// * `current-char` (get,set) - The current char of the RollingChar
    pub struct RollingChar(ObjectSubclass<imp::RollingCharPriv>)
        @extends gtk::Widget,
        @implements gtk::Buildable;
}

impl Default for RollingChar {
    fn default() -> Self {
        let sel = glib::Object::new::<Self>();
        sel.set_overflow(gtk::Overflow::Hidden);
        sel
    }
}

impl RollingChar {
    pub fn new(char: Option<char>) -> Self {
        let rolling_num = Self::default();
        rolling_num.imp().current_char.replace(char.unwrap_or('-'));
        rolling_num
            .imp()
            .primary_label
            .borrow()
            .set_text(char.unwrap_or('-').to_string().as_str());
        rolling_num
            .imp()
            .secondary_label
            .borrow()
            .set_text(char.unwrap_or('-').to_string().as_str());
        rolling_num
    }
}
