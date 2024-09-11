pub mod imp;
pub mod local_css_context;
mod object_subclass_impl;
use abi::{glib, gtk};

glib::wrapper! {
    /// A Label with a max width (`width_request`) that scrolls when the inner label exceeds the max width
    ///
    /// # Properties
    /// * `label` (get,set) - The label widget used to display the text, only use this to set properties of the label
    /// * `text` (get,set) - The text to be displayed
    /// * `active` (get) - Whether the scrolling is active(the text exceeds the max width)
    /// * `config-fade-size` (get,set) - The size of the fade on the sides of the label, can be a percentage(example: "5%") or a pixel value(example: "5px" or "5")
    /// * `config-scroll-speed` (get,set) - The speed of the scrolling in pixels per second
    /// * `max-width` (get,set) - The maximum width of the label
    /// * `config-delay` (get,set) - The time in milliseconds from when the scrolling stops to when it starts again
    pub struct ScrollingLabel(ObjectSubclass<imp::ScrollingLabelPriv>)
        @extends gtk::Widget,
        @implements gtk::Buildable;
}

impl Default for ScrollingLabel {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

impl ScrollingLabel {
    pub fn new() -> Self {
        Self::default()
    }
}
