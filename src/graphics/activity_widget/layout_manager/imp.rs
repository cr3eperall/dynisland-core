use abi::{
    gtk::{self, prelude::*, subclass::prelude::*},
    log,
};

use crate::graphics::activity_widget::{
    boxed_activity_mode::ActivityMode, imp::ActivityWidgetPriv, util, ActivityWidget,
};

#[derive(Default)]
pub struct ActivityLayoutManagerPriv {}

impl ObjectImpl for ActivityLayoutManagerPriv {}

// Force the size of the ActivityWidget to be the one of the background widget, so that it can be controlled by css and it can be animated
impl LayoutManagerImpl for ActivityLayoutManagerPriv {
    fn measure(
        &self,
        widget: &gtk::Widget,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        let activity_widget = widget.downcast_ref::<ActivityWidget>();
        if activity_widget.is_none() {
            log::error!("Error downcasting ActivityWidget");
            return (0, 0, -1, -1);
        }
        let activity_widget = activity_widget.unwrap();

        let min_height = activity_widget.config_minimal_height();
        if activity_widget.has_css_class("hidden") && orientation == gtk::Orientation::Horizontal {
            return (0, 0, -1, -1);
        }
        if !activity_widget.has_css_class("dragging") {
            let next_size = ActivityWidgetPriv::get_final_widget_size_for_mode(
                activity_widget,
                activity_widget.mode(),
                min_height,
                activity_widget.config_minimal_width(),
            );
            let mut css_context = activity_widget.imp().local_css_context.borrow_mut();
            css_context.set_size((next_size.0 as i32, next_size.1 as i32));
        }
        let first_child = activity_widget.first_child(); //should be the background widget
        match first_child {
            Some(first_child) => {
                let (min_size, nat_size, _, _) = first_child.measure(orientation, for_size);
                (min_height.max(min_size), min_height.max(nat_size), -1, -1)
            }
            None => (min_height, min_height, -1, -1),
        }
    }

    fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, baseline: i32) {
        let activity_widget = widget.downcast_ref::<ActivityWidget>();
        if activity_widget.is_none() {
            log::error!("Error downcasting ActivityWidget");
            return;
        }
        let activity_widget = activity_widget.unwrap();
        let min_height = activity_widget.config_minimal_height();
        let activity = activity_widget.imp();

        if let Some(content) = &*activity.background_widget.borrow() {
            content.allocate(width, height, -1, None);
        };

        if let Some(content) = &*activity.minimal_mode_widget.borrow() {
            let (width, height, transform) = util::get_child_aligned_allocation(
                (width, height, baseline),
                content,
                ActivityMode::Minimal,
                min_height,
                activity_widget.has_css_class("dragging"),
            );

            content.allocate(width, height, -1, transform);
        }
        if let Some(content) = &*activity.compact_mode_widget.borrow() {
            let (width, height, transform) = util::get_child_aligned_allocation(
                (width, height, baseline),
                content,
                ActivityMode::Compact,
                min_height,
                activity_widget.has_css_class("dragging"),
            );

            content.allocate(width, height, -1, transform);
        }
        if let Some(content) = &*activity.expanded_mode_widget.borrow() {
            let (width, height, transform) = util::get_child_aligned_allocation(
                (width, height, baseline),
                content,
                ActivityMode::Expanded,
                min_height,
                activity_widget.has_css_class("dragging"),
            );

            content.allocate(width, height, -1, transform);
        }
        if let Some(content) = &*activity.overlay_mode_widget.borrow() {
            let (width, height, transform) = util::get_child_aligned_allocation(
                (width, height, baseline),
                content,
                ActivityMode::Overlay,
                min_height,
                activity_widget.has_css_class("dragging"),
            );

            content.allocate(width, height, -1, transform);
        };
    }
}
