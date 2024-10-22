use std::cell::RefCell;

use abi::{gdk, glib, glib_macros, gtk, log};
use glib::prelude::*;
use glib_macros::Properties;
use gtk::{prelude::*, subclass::prelude::*, StateFlags};
use rand::{distributions::Alphanumeric, Rng};

use super::{
    boxed_activity_mode::ActivityMode, local_css_context::ActivityWidgetLocalCssContext, util,
    ActivityWidget,
};

#[derive(Properties)]
#[properties(wrapper_type = ActivityWidget)]
pub struct ActivityWidgetPriv {
    #[property(get, set, nick = "Change mode", blurb = "The Activity Mode")]
    pub(super) mode: RefCell<ActivityMode>,

    // #[property(get, nick = "Local CSS Context")]
    pub(super) local_css_context: RefCell<ActivityWidgetLocalCssContext>,

    #[property(get, set, nick = "Widget name")]
    pub(super) name: RefCell<String>,

    /// To be used by dynisland::app and layout managers only
    #[property(get, set, nick = "Minimal height")]
    pub(super) config_minimal_height: RefCell<i32>,

    /// To be used by dynisland::app and layout managers only
    #[property(get, set, nick = "Minimal width")]
    pub(super) config_minimal_width: RefCell<i32>,

    /// To be used by dynisland::app and layout managers only
    #[property(get, set, nick = "Transition blur radius")]
    pub(super) config_blur_radius: RefCell<f64>,

    /// To be used by dynisland::app and layout managers only
    #[property(get, set, nick = "Enable stretching on drag")]
    pub(super) config_enable_drag_stretch: RefCell<bool>,

    #[property(get, nick = "The Last Activity mode")]
    pub(super) last_mode: RefCell<ActivityMode>,

    // pub(super) transition_manager: RefCell<TransitionManager>,
    pub(super) background_widget: RefCell<Option<gtk::Widget>>,

    #[property(get, set, nick = "Minimal Mode Widget")]
    pub(super) minimal_mode_widget: RefCell<Option<gtk::Widget>>,
    #[property(get, set, nick = "Compact Mode Widget")]
    pub(super) compact_mode_widget: RefCell<Option<gtk::Widget>>,
    #[property(get, set, nick = "Expanded Mode Widget")]
    pub(super) expanded_mode_widget: RefCell<Option<gtk::Widget>>,
    #[property(get, set, nick = "Overlay Mode Widget")]
    pub(super) overlay_mode_widget: RefCell<Option<gtk::Widget>>,
}

//default data
impl Default for ActivityWidgetPriv {
    fn default() -> Self {
        let name: String = "c"
            .chars()
            .chain(
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(6)
                    .map(char::from),
            )
            .collect::<String>();

        let css_ctx = ActivityWidgetLocalCssContext::new(&name);
        let min_h = 40;
        let min_w = 60;
        let blur = 6.0;
        let enable_stretch = false;
        Self {
            mode: RefCell::new(ActivityMode::Minimal),
            local_css_context: RefCell::new(css_ctx),
            config_minimal_height: RefCell::new(min_h),
            config_minimal_width: RefCell::new(min_w),
            config_blur_radius: RefCell::new(blur),
            config_enable_drag_stretch: RefCell::new(enable_stretch),
            last_mode: RefCell::new(ActivityMode::Minimal),
            name: RefCell::new(name),
            minimal_mode_widget: RefCell::new(None),
            compact_mode_widget: RefCell::new(None),
            expanded_mode_widget: RefCell::new(None),
            overlay_mode_widget: RefCell::new(None),
            background_widget: RefCell::new(None),
        }
    }
}

#[glib::derived_properties]
impl ObjectImpl for ActivityWidgetPriv {
    fn constructed(&self) {
        self.parent_constructed();
        let background = gtk::Box::builder()
            .valign(gtk::Align::Start)
            .halign(gtk::Align::Center)
            .vexpand(true)
            .hexpand(true)
            .build();
        background.add_css_class("activity-background");

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
            self.local_css_context.borrow().get_css_provider(),
            gtk::STYLE_PROVIDER_PRIORITY_USER + 1, //needs to be higher than user proprity
        );

        self.add_drag_controller();

        background.set_parent(&*self.obj());
        self.background_widget
            .replace(Some(background.upcast::<gtk::Widget>()));
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "mode" => {
                // Replace old values if the mode is valid
                let mode = value.get().unwrap();
                let obj = self.obj();

                if self.get_mode_widget(mode).borrow().is_none() {
                    return;
                }
                if let Some(prev) = self
                    .get_mode_widget(*self.last_mode.borrow())
                    .borrow()
                    .as_ref()
                {
                    prev.remove_css_class("prev");
                    prev.remove_css_class("next");
                }
                match mode {
                    ActivityMode::Minimal => {
                        obj.add_css_class("in-minimal-mode");
                        obj.remove_css_class("in-compact-mode");
                        obj.remove_css_class("in-expanded-mode");
                        obj.remove_css_class("in-overlay-mode");
                    }
                    ActivityMode::Compact => {
                        obj.remove_css_class("in-minimal-mode");
                        obj.add_css_class("in-compact-mode");
                        obj.remove_css_class("in-expanded-mode");
                        obj.remove_css_class("in-overlay-mode");
                    }
                    ActivityMode::Expanded => {
                        obj.remove_css_class("in-minimal-mode");
                        obj.remove_css_class("in-compact-mode");
                        obj.add_css_class("in-expanded-mode");
                        obj.remove_css_class("in-overlay-mode");
                    }
                    ActivityMode::Overlay => {
                        obj.remove_css_class("in-minimal-mode");
                        obj.remove_css_class("in-compact-mode");
                        obj.remove_css_class("in-expanded-mode");
                        obj.add_css_class("in-overlay-mode");
                    }
                }

                self.last_mode.replace(*self.mode.borrow());
                self.mode.replace(mode);

                // let last_mode = *self.last_mode.borrow();

                let mut css_context = self.local_css_context.borrow_mut();
                let min_height = *self.config_minimal_height.borrow();
                let min_width = *self.config_minimal_width.borrow();

                let next_size =
                    Self::get_final_widget_size_for_mode(&obj, mode, min_height, min_width);
                // log::debug!("next_size: {:?}", next_size);
                // let prev_size=self.get_final_allocation_for_mode(last_mode, min_height);

                // TODO add css classes {active, bigger, smaller, last...} to the widgets accordingly
                // let bigger = next_size.0 > prev_size.0 || next_size.1 > prev_size.1;

                // Set properties to start the css transition

                css_context.set_opacity_all(util::get_property_slice_for_mode_f64(mode, 1.0, 0.0));

                let blur_radius = *self.config_blur_radius.borrow();
                css_context.set_blur_all(util::get_property_slice_for_mode_f64(
                    mode,
                    0.0,
                    blur_radius,
                ));

                let stretches = Self::get_stretches(&obj, next_size, min_height, min_width);
                log::trace!("stretches: {:?}", stretches);
                css_context.set_stretch_all(stretches, None);

                if let Some(next) = self.get_mode_widget(mode).borrow().as_ref() {
                    next.remove_css_class("prev");
                    next.add_css_class("next");
                    next.set_visible(true);
                    //put at the end so it receives the inputs
                    next.insert_before(self.obj().as_ref(), Option::None::<&gtk::Widget>);
                    css_context.set_size((next_size.0 as i32, next_size.1 as i32));
                };
                if let Some(prev) = self
                    .get_mode_widget(*self.last_mode.borrow())
                    .borrow()
                    .as_ref()
                {
                    prev.remove_css_class("next");
                    prev.add_css_class("prev");
                }
                self.obj().queue_draw(); // Queue a draw call with the updated value
            }
            "name" => {
                self.obj().remove_css_class(&self.name.borrow());

                self.name.replace(value.get().unwrap());
                self.local_css_context
                    .borrow_mut()
                    .set_name(value.get().unwrap());
                self.obj().add_css_class(value.get().unwrap());
            }
            "config-minimal-height" => {
                let height = value.get().unwrap();
                self.config_minimal_height.replace(height);
                self.local_css_context
                    .borrow_mut()
                    .set_config_minimal_height(value.get().unwrap());
            }
            "config-minimal-width" => {
                let width = value.get().unwrap();
                self.config_minimal_width.replace(width);
            }
            "config-blur-radius" => {
                self.config_blur_radius.replace(value.get().unwrap());
            }
            "config-enable-drag-stretch" => {
                self.config_enable_drag_stretch
                    .replace(value.get().unwrap());
            }
            "minimal-mode-widget" => {
                let widget: Option<gtk::Widget> = value.get().unwrap();
                if let Some(content) = &*self.minimal_mode_widget.borrow() {
                    content.unparent();
                    content.remove_css_class("mode-minimal");
                }
                self.minimal_mode_widget.replace(widget);
                if let Some(widget) = self.minimal_mode_widget.borrow().as_ref() {
                    widget.set_parent(self.obj().upcast_ref::<gtk::Widget>());
                    widget.add_css_class("mode-minimal");
                    widget.set_overflow(gtk::Overflow::Hidden);
                }
                self.obj().set_mode(self.obj().mode()); //update the size and the position of the widget
                self.obj().queue_draw(); // Queue a draw call with the updated widget
            }
            "compact-mode-widget" => {
                let widget: Option<gtk::Widget> = value.get().unwrap();
                if let Some(content) = &*self.compact_mode_widget.borrow() {
                    content.unparent();
                    content.remove_css_class("mode-compact");
                }
                self.compact_mode_widget.replace(widget);
                if let Some(widget) = self.compact_mode_widget.borrow().as_ref() {
                    widget.set_parent(self.obj().upcast_ref::<gtk::Widget>());
                    widget.add_css_class("mode-compact");
                    widget.set_overflow(gtk::Overflow::Hidden);
                }

                self.obj().set_mode(self.obj().mode()); //update the size and the position of the widget
                self.obj().queue_draw(); // Queue a draw call with the updated widget
            }
            "expanded-mode-widget" => {
                let widget: Option<gtk::Widget> = value.get().unwrap();
                if let Some(content) = &*self.expanded_mode_widget.borrow() {
                    content.unparent();
                    content.remove_css_class("mode-expanded");
                }
                self.expanded_mode_widget.replace(widget);
                if let Some(widget) = self.expanded_mode_widget.borrow().as_ref() {
                    widget.set_parent(self.obj().upcast_ref::<gtk::Widget>());
                    widget.add_css_class("mode-expanded");
                    widget.set_overflow(gtk::Overflow::Hidden);
                }

                self.obj().set_mode(self.obj().mode()); //update the size and the position of the widget
                self.obj().queue_draw(); // Queue a draw call with the updated widget
            }
            "overlay-mode-widget" => {
                let widget: Option<gtk::Widget> = value.get().unwrap();
                if let Some(content) = &*self.overlay_mode_widget.borrow() {
                    content.unparent();
                    content.remove_css_class("mode-overlay");
                }
                self.overlay_mode_widget.replace(widget);
                if let Some(widget) = self.overlay_mode_widget.borrow().as_ref() {
                    widget.set_parent(self.obj().upcast_ref::<gtk::Widget>());
                    widget.add_css_class("mode-overlay");
                    widget.set_overflow(gtk::Overflow::Hidden);
                }

                self.obj().set_mode(self.obj().mode()); //update the size and the position of the widget
                self.obj().queue_draw(); // Queue a draw call with the updated widget
            }

            x => panic!("Tried to set inexistant property of ActivityWidget: {}", x),
        }
    }

    fn dispose(&self) {
        // log::warn!("{} dispose", self.name.borrow());
        if let Some(widget) = self.background_widget.borrow_mut().take() {
            widget.unparent();
        }
        if let Some(widget) = self.minimal_mode_widget.borrow_mut().take() {
            widget.unparent();
        }
        if let Some(widget) = self.compact_mode_widget.borrow_mut().take() {
            widget.unparent();
        }
        if let Some(widget) = self.expanded_mode_widget.borrow_mut().take() {
            widget.unparent();
        }
        if let Some(widget) = self.overlay_mode_widget.borrow_mut().take() {
            widget.unparent();
        }
    }
}

impl WidgetImpl for ActivityWidgetPriv {}

impl ActivityWidgetPriv {
    fn add_drag_controller(&self) {
        //TODO add configurable scaling factor / log function for stretching
        let drag_controller = gtk::GestureDrag::builder()
            .button(gdk::BUTTON_PRIMARY)
            .name("drag-gesture")
            .build();
        drag_controller.connect_drag_begin(|gest, _, _| {
            let obj = gest.widget().downcast::<ActivityWidget>().unwrap();
            if !obj.config_enable_drag_stretch() {
                return;
            }
            obj.add_css_class("dragging");
        });
        drag_controller.connect_drag_update(|gest, x, y| {
            let obj = gest.widget().downcast::<ActivityWidget>().unwrap();
            // log::info!("enable stretch: {}", obj.local_css_context().get_config_enable_drag_stretch());
            if !obj.config_enable_drag_stretch() {
                return;
            }
            let min_height = obj.config_minimal_height();
            let min_width = obj.config_minimal_width();
            let starting_size =
                Self::get_final_widget_size_for_mode(&obj, obj.mode(), min_height, min_width);
            let x = if gest.start_point().unwrap().0 < starting_size.0 / 2.0 {
                -x
            } else {
                x
            };
            let y = if gest.start_point().unwrap().1 < starting_size.1 / 2.0 {
                -y
            } else {
                y
            };
            // log::info!("{:?} {:?}",start, starting_size);
            // log::info!("continue: {x} {y}");
            let current_size =
                Self::get_final_widget_size_for_mode(&obj, obj.mode(), min_height, min_width);
            let max_screen_size = util::get_max_monitors_size();
            // log::info!("max: {:?}", max_screen_size);
            let next_size = (
                (current_size.0 * (1.0 + (x / max_screen_size.0 as f64)))
                    .max(obj.config_minimal_width() as f64),
                (current_size.1 * (1.0 + (y / max_screen_size.1 as f64)))
                    .max(obj.config_minimal_height() as f64),
            );
            let mut stretches = Self::get_stretches(&obj, next_size, min_height, min_width);
            let current_stretch = (next_size.0 / starting_size.0, next_size.1 / starting_size.1);
            stretches[obj.mode() as usize] = current_stretch;
            // log::trace!("stretches: {:?}", stretches);
            let mut css_context = obj.imp().local_css_context.borrow_mut();
            let translates = Self::get_translates(&obj, next_size, stretches, min_height);
            css_context.set_stretch_all(stretches, Some(translates));

            css_context.set_size((next_size.0 as i32, next_size.1 as i32));
            obj.queue_draw();
        });
        drag_controller.connect_drag_end(|gest, _, _| {
            let obj = gest.widget().downcast::<ActivityWidget>().unwrap();
            if !obj.config_enable_drag_stretch() {
                return;
            }
            if obj.has_css_class("dragging") {
                obj.remove_css_class("dragging");
                let min_height = obj.config_minimal_height();
                let min_width = obj.config_minimal_width();
                let next_size =
                    Self::get_final_widget_size_for_mode(&obj, obj.mode(), min_height, min_width);
                let stretches = Self::get_stretches(&obj, next_size, min_height, min_width);
                // log::trace!("stretches: {:?}", stretches);
                let mut css_context = obj.imp().local_css_context.borrow_mut();
                css_context.set_stretch_all(stretches, None);

                css_context.set_size((next_size.0 as i32, next_size.1 as i32));
                obj.queue_draw();
            }
        });
        self.obj().connect_state_flags_changed(|obj, _| {
            if !obj.config_enable_drag_stretch() {
                return;
            }
            if obj.has_css_class("dragging") && !obj.state_flags().contains(StateFlags::ACTIVE) {
                obj.remove_css_class("dragging");
                let min_height = obj.config_minimal_height();
                let min_width = obj.config_minimal_width();
                let next_size =
                    Self::get_final_widget_size_for_mode(obj, obj.mode(), min_height, min_width);
                let stretches = Self::get_stretches(obj, next_size, min_height, min_width);
                // log::trace!("stretches: {:?}", stretches);
                let mut css_context = obj.imp().local_css_context.borrow_mut();
                css_context.set_stretch_all(stretches, None);

                css_context.set_size((next_size.0 as i32, next_size.1 as i32));
                obj.queue_draw();
            }
        });
        self.obj().add_controller(drag_controller);
    }
    pub(super) fn get_mode_widget(&self, mode: ActivityMode) -> &RefCell<Option<gtk::Widget>> {
        match mode {
            ActivityMode::Minimal => &self.minimal_mode_widget,
            ActivityMode::Compact => &self.compact_mode_widget,
            ActivityMode::Expanded => &self.expanded_mode_widget,
            ActivityMode::Overlay => &self.overlay_mode_widget,
        }
    }

    pub(super) fn get_final_widget_size_for_mode(
        obj: &ActivityWidget,
        mode: ActivityMode,
        min_height: i32,
        min_width: i32,
    ) -> (f64, f64) {
        if let Some(widget) = &obj.get_widget_for_mode(mode) {
            let tmp = util::get_final_widget_size(widget, obj.mode(), min_height, min_width);
            (tmp.0 as f64, tmp.1 as f64)
        } else {
            (
                // default is the current size
                obj.width() as f64,
                obj.height() as f64,
            )
        }
    }

    pub(super) fn get_stretches(
        obj: &ActivityWidget,
        next_size: (f64, f64),
        min_height: i32,
        min_width: i32,
    ) -> [(f64, f64); 4] {
        let mut mode = ActivityMode::Minimal;
        let min_stretch = if matches!(obj.mode(), ActivityMode::Minimal) {
            (1.0, 1.0)
        } else {
            let min_alloc = if let Some(widget) = &obj.get_widget_for_mode(mode) {
                let mut measure = util::get_child_aligned_allocation(
                    (next_size.0 as i32, next_size.1 as i32, -1),
                    widget,
                    mode,
                    min_height,
                    false,
                );
                if measure.0 == 0 {
                    measure.0 = next_size.0 as i32;
                }
                if measure.1 == 0 {
                    measure.1 = next_size.1 as i32;
                }
                (measure.0 as f64, measure.1 as f64)
            } else {
                Self::get_final_widget_size_for_mode(obj, mode, min_height, min_width)
            };
            // log::debug!("min get_size: {:?}, alloc: {:?}", min_alloc, min_alloc);
            (next_size.0 / min_alloc.0, next_size.1 / min_alloc.1)
        };

        mode = ActivityMode::Compact;
        let com_stretch = if matches!(obj.mode(), ActivityMode::Compact) {
            (1.0, 1.0)
        } else {
            let com_alloc = if let Some(widget) = &obj.get_widget_for_mode(mode) {
                let mut measure = util::get_child_aligned_allocation(
                    (next_size.0 as i32, next_size.1 as i32, -1),
                    widget,
                    mode,
                    min_height,
                    false,
                );
                if measure.0 == 0 {
                    measure.0 = next_size.0 as i32;
                }
                if measure.1 == 0 {
                    measure.1 = next_size.1 as i32;
                }
                (measure.0 as f64, measure.1 as f64)
            } else {
                Self::get_final_widget_size_for_mode(obj, mode, min_height, min_width)
            };
            // log::debug!("min get_size: {:?}, alloc: {:?}", min_alloc, min_alloc);
            (next_size.0 / com_alloc.0, next_size.1 / com_alloc.1)
        };

        mode = ActivityMode::Expanded;
        let exp_stretch = if matches!(obj.mode(), ActivityMode::Expanded) {
            (1.0, 1.0)
        } else {
            let exp_alloc = if let Some(widget) = &obj.get_widget_for_mode(mode) {
                let mut measure = util::get_child_aligned_allocation(
                    (next_size.0 as i32, next_size.1 as i32, -1),
                    widget,
                    mode,
                    min_height,
                    false,
                );
                if measure.0 == 0 {
                    measure.0 = next_size.0 as i32;
                }
                if measure.1 == 0 {
                    measure.1 = next_size.1 as i32;
                }
                (measure.0 as f64, measure.1 as f64)
            } else {
                Self::get_final_widget_size_for_mode(obj, mode, min_height, min_width)
            };
            // log::debug!("min get_size: {:?}, alloc: {:?}", min_alloc, min_alloc);
            (next_size.0 / exp_alloc.0, next_size.1 / exp_alloc.1)
        };

        mode = ActivityMode::Overlay;
        let ove_stretch = if matches!(obj.mode(), ActivityMode::Overlay) {
            (1.0, 1.0)
        } else {
            let ove_alloc = if let Some(widget) = &obj.get_widget_for_mode(mode) {
                let mut measure = util::get_child_aligned_allocation(
                    (next_size.0 as i32, next_size.1 as i32, -1),
                    widget,
                    mode,
                    min_height,
                    false,
                );
                if measure.0 == 0 {
                    measure.0 = next_size.0 as i32;
                }
                if measure.1 == 0 {
                    measure.1 = next_size.1 as i32;
                }
                (measure.0 as f64, measure.1 as f64)
            } else {
                Self::get_final_widget_size_for_mode(obj, mode, min_height, min_width)
            };
            // log::debug!("min get_size: {:?}, alloc: {:?}", min_alloc, min_alloc);
            (next_size.0 / ove_alloc.0, next_size.1 / ove_alloc.1)
        };

        [
            (min_stretch.0, min_stretch.1),
            (com_stretch.0, com_stretch.1),
            (exp_stretch.0, exp_stretch.1),
            (ove_stretch.0, ove_stretch.1),
        ]
    }

    pub(super) fn get_translates(
        obj: &ActivityWidget,
        next_size: (f64, f64),
        stretches: [(f64, f64); 4],
        min_height: i32,
    ) -> [(f64, f64); 4] {
        let modes = [
            ActivityMode::Minimal,
            ActivityMode::Compact,
            ActivityMode::Expanded,
            ActivityMode::Overlay,
        ];
        let mut translates = [(0.0, 0.0); 4];
        for (i, mode) in modes.into_iter().enumerate() {
            let translate = if let Some(widget) = obj.get_widget_for_mode(mode) {
                let measure = util::get_child_aligned_allocation(
                    (next_size.0 as i32, next_size.1 as i32, -1),
                    &widget,
                    mode,
                    min_height,
                    false,
                );
                let widget_width = measure.0 as f64;
                let widget_height = measure.1 as f64;
                let x = match widget.halign() {
                    gtk::Align::Start => (next_size.0 - widget_width) / 2.0,
                    gtk::Align::End => -(next_size.0 - widget_width as f64) / 2.0,
                    gtk::Align::Fill => {
                        if widget_width > next_size.0 {
                            0.0
                        } else {
                            -(next_size.0 - widget_width as f64 * stretches[i].0) / 2.0
                        }
                    }
                    _ => {
                        // center
                        0.0
                    }
                };
                let y = match widget.valign() {
                    gtk::Align::Start => (next_size.1 - widget_height as f64) / 2.0,
                    gtk::Align::End => -(next_size.1 - widget_height as f64) / 2.0,
                    gtk::Align::Fill => {
                        if widget_height > next_size.1 {
                            0.0
                        } else {
                            -(next_size.1 - widget_height as f64 * stretches[i].1) / 2.0
                        }
                    }
                    _ => {
                        // center
                        0.0
                    }
                };
                (x, y)
            } else {
                (0.0, 0.0)
            };
            translates[i] = translate;
            if mode == ActivityMode::Expanded {
                translates[i] = (translate.0 / stretches[i].0, translate.1 / stretches[i].1);
                // log::debug!(
                //     "translate: {:?}, stretch: {:?}",
                //     translates[i],
                //     stretches[i]
                // );
            }
        }

        translates
    }
}
