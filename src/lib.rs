pub mod activity_map;
pub mod base_module;
pub mod config_variable;
pub mod dynamic_activity;
pub mod dynamic_property;
pub mod graphics;

pub extern crate dynisland_abi as abi;
pub extern crate ron;
// pub extern crate grass;

#[macro_export]
macro_rules! randomize_name {
    ($name:literal) => {
        std::concat!($name, "_", const_random::const_random!(u16))
    };
}
