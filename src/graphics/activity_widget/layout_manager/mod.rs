mod imp;
use abi::gtk;

abi::glib::wrapper! {
    pub struct ActivityLayoutManager(ObjectSubclass<imp::ActivityLayoutManagerPriv>)
        @extends gtk::LayoutManager;
}
