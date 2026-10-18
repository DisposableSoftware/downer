//! A single-child container that keeps its child at most `MAX` px wide and centred,
//! with `PAD` px around it (`VIEW-2`). GTK has no max-width, and `AdwClamp` would need
//! libadwaita.

use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

const MAX: i32 = 800;
const PAD: i32 = 24;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Column {
        pub after_allocate: RefCell<Option<Box<dyn FnOnce()>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Column {
        const NAME: &'static str = "DownerColumn";
        type Type = super::Column;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for Column {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Column {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let Some(child) = self.obj().first_child() else {
                return (0, 0, -1, -1);
            };
            if orientation == gtk::Orientation::Horizontal {
                let (min, _, _, _) = child.measure(orientation, -1);
                (min + 2 * PAD, min.max(MAX) + 2 * PAD, -1, -1)
            } else {
                let width = if for_size < 0 {
                    -1
                } else {
                    child_width(&child, for_size)
                };
                let (min, nat, _, _) = child.measure(orientation, width);
                (min + 2 * PAD, nat + 2 * PAD, -1, -1)
            }
        }

        fn size_allocate(&self, width: i32, height: i32, _baseline: i32) {
            let Some(child) = self.obj().first_child() else {
                return;
            };
            let w = child_width(&child, width);
            let transform = gtk::gsk::Transform::new().translate(&gtk::graphene::Point::new(
                ((width - w) / 2) as f32,
                PAD as f32,
            ));
            child.allocate(w, (height - 2 * PAD).max(0), -1, Some(transform));
            if let Some(f) = self.after_allocate.take() {
                f();
            }
        }
    }
}

fn child_width(child: &gtk::Widget, width: i32) -> i32 {
    let (min, _, _, _) = child.measure(gtk::Orientation::Horizontal, -1);
    (width - 2 * PAD).min(MAX).max(min)
}

glib::wrapper! {
    pub struct Column(ObjectSubclass<imp::Column>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Column {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Runs  once, after the next time the child is laid out.
    pub fn after_next_allocate(&self, f: impl FnOnce() + 'static) {
        self.imp().after_allocate.replace(Some(Box::new(f)));
    }

    pub fn set_child(&self, child: &impl IsA<gtk::Widget>) {
        while let Some(old) = self.first_child() {
            old.unparent();
        }
        child.set_parent(self);
    }
}
