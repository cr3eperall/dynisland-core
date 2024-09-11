mod imp;
mod object_subclass_impl;
use abi::gtk;

abi::glib::wrapper! {
    pub struct ActivityLayoutManager(ObjectSubclass<imp::ActivityLayoutManagerPriv>)
        @extends gtk::LayoutManager;
}
