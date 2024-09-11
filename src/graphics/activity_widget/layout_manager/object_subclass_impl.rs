use std::ffi::CString;

use abi::{glib, gtk};
use glib::{ffi::GType, subclass::types::*, translate::FromGlib};

use super::imp::ActivityLayoutManagerPriv;

// Recursive expansion of object_subclass macro
// =============================================

impl ObjectSubclass for ActivityLayoutManagerPriv {
    type Interfaces = ();
    type Class = glib::subclass::basic::ClassStruct<Self>;
    type Instance = glib::subclass::basic::InstanceStruct<Self>;
    #[inline]
    fn new() -> Self {
        ::std::default::Default::default()
    }
    const NAME: &'static str = "ActivityLayoutManager";
    type Type = super::ActivityLayoutManager;
    type ParentType = gtk::LayoutManager;
}
unsafe impl glib::subclass::types::ObjectSubclassType for ActivityLayoutManagerPriv {
    #[inline]
    fn type_data() -> ::std::ptr::NonNull<glib::subclass::TypeData> {
        static mut DATA: glib::subclass::TypeData = glib::subclass::types::INIT_TYPE_DATA;
        unsafe { ::std::ptr::NonNull::from(&mut DATA) }
    }
    #[inline]
    fn type_() -> glib::Type {
        static ONCE: ::std::sync::Once = ::std::sync::Once::new();
        ONCE.call_once(|| {
            unsafe {
                let type_name = CString::new(Self::NAME).unwrap();
                let gtype: GType = glib::gobject_ffi::g_type_from_name(type_name.as_ptr());

                if gtype == glib::gobject_ffi::G_TYPE_INVALID {
                    // type needs to be registered
                    glib::subclass::register_type::<Self>();
                } else {
                    // type was already registered by another module, it should be safe to not register it
                    let type_ = glib::Type::from_glib(gtype);
                    let mut data = Self::type_data();
                    data.as_mut().type_ = type_;
                    //FIXME set other type data like private_offset, this could cause crashes in the future
                    // data.as_mut().private_offset = std::mem::size_of::<glib::subclass::types::PrivateStruct<Self>>()
                }
            }
        });
        unsafe {
            let data = Self::type_data();
            let type_ = data.as_ref().type_();
            type_
        }
    }
}
#[doc(hidden)]
impl glib::subclass::types::FromObject for ActivityLayoutManagerPriv {
    type FromObjectType = <Self as glib::subclass::types::ObjectSubclass>::Type;
    #[inline]
    fn from_object(obj: &Self::FromObjectType) -> &Self {
        <Self as glib::subclass::types::ObjectSubclassExt>::from_obj(obj)
    }
}
#[doc(hidden)]
impl glib::clone::Downgrade for ActivityLayoutManagerPriv {
    type Weak = glib::subclass::ObjectImplWeakRef<ActivityLayoutManagerPriv>;
    #[inline]
    fn downgrade(&self) -> Self::Weak {
        let ref_counted = glib::subclass::prelude::ObjectSubclassExt::ref_counted(self);
        glib::clone::Downgrade::downgrade(&ref_counted)
    }
}
impl ActivityLayoutManagerPriv {
    #[inline]
    pub fn downgrade(&self) -> <Self as glib::clone::Downgrade>::Weak {
        glib::clone::Downgrade::downgrade(self)
    }
}
#[doc(hidden)]
impl ::std::borrow::ToOwned for ActivityLayoutManagerPriv {
    type Owned = glib::subclass::ObjectImplRef<ActivityLayoutManagerPriv>;
    #[inline]
    fn to_owned(&self) -> Self::Owned {
        glib::subclass::prelude::ObjectSubclassExt::ref_counted(self)
    }
}
#[doc(hidden)]
impl ::std::borrow::Borrow<ActivityLayoutManagerPriv>
    for glib::subclass::ObjectImplRef<ActivityLayoutManagerPriv>
{
    #[inline]
    fn borrow(&self) -> &ActivityLayoutManagerPriv {
        self
    }
}
