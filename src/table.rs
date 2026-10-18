//! A table container whose cells wrap (`VIEW-10`). `GtkGrid` reports a minimum height
//! at its minimum width that is taller than its height at the width it gets, and GTK
//! warns and leaves spare height. Here one function turns a width into column widths
//! and row heights, and `measure` answers every question from it.

use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Table {
        /// The row and column of each child, in the order the children were added.
        pub cells: RefCell<Vec<(usize, usize)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Table {
        const NAME: &'static str = "DownerTable";
        type Type = super::Table;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for Table {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Table {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let cells = self.cells();
            let (min, nat) = column_widths(&cells);
            let (min_total, nat_total) = (min.iter().sum(), nat.iter().sum());
            let height_at =
                |width: i32| -> i32 { row_heights(&cells, &share(&min, &nat, width)).iter().sum() };
            if orientation == gtk::Orientation::Horizontal {
                if for_size < 0 {
                    return (min_total, nat_total, -1, -1);
                }
                // The narrowest width that is no taller than `for_size`; heights only
                // fall as the width grows.
                let (mut lo, mut hi) = (min_total, nat_total);
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    if height_at(mid) <= for_size {
                        hi = mid;
                    } else {
                        lo = mid + 1;
                    }
                }
                return (lo, nat_total, -1, -1);
            }
            // The natural width gives the shortest height, so it is the overall minimum;
            // a narrower width only ever needs more.
            let width = if for_size < 0 { nat_total } else { for_size };
            let height = height_at(width);
            (height, height, -1, -1)
        }

        fn size_allocate(&self, width: i32, _height: i32, _baseline: i32) {
            let cells = self.cells();
            let (min, nat) = column_widths(&cells);
            let widths = share(&min, &nat, width);
            let heights = row_heights(&cells, &widths);
            for (child, r, c) in &cells {
                let x: i32 = widths[..*c].iter().sum();
                let y: i32 = heights[..*r].iter().sum();
                let transform = gtk::gsk::Transform::new()
                    .translate(&gtk::graphene::Point::new(x as f32, y as f32));
                child.allocate(widths[*c], heights[*r], -1, Some(transform));
            }
        }
    }

    impl Table {
        fn cells(&self) -> Vec<(gtk::Widget, usize, usize)> {
            let mut out = Vec::new();
            let mut child = self.obj().first_child();
            for &(r, c) in self.cells.borrow().iter() {
                let Some(w) = child else { break };
                child = w.next_sibling();
                out.push((w, r, c));
            }
            out
        }
    }
}

type Cells = [(gtk::Widget, usize, usize)];

/// The minimum and natural width of each column: the widest cell in it.
fn column_widths(cells: &Cells) -> (Vec<i32>, Vec<i32>) {
    let n = cells.iter().map(|(_, _, c)| c + 1).max().unwrap_or(0);
    let (mut min, mut nat) = (vec![0; n], vec![0; n]);
    for (child, _, c) in cells {
        let (m, n, _, _) = child.measure(gtk::Orientation::Horizontal, -1);
        min[*c] = min[*c].max(m);
        nat[*c] = nat[*c].max(n);
    }
    (min, nat)
}

/// Column widths for a table `width` wide: each column's minimum, plus a share of the
/// spare width in proportion to how much more it would like. Never wider than natural.
fn share(min: &[i32], nat: &[i32], width: i32) -> Vec<i32> {
    let min_total: i32 = min.iter().sum();
    let want: i32 = nat.iter().zip(min).map(|(n, m)| n - m).sum();
    let extra = (width - min_total).clamp(0, want);
    let mut left = extra;
    let mut out = Vec::with_capacity(min.len());
    for (i, (n, m)) in nat.iter().zip(min).enumerate() {
        let give = if i + 1 == min.len() || want == 0 {
            left.min(n - m)
        } else {
            (extra as i64 * (n - m) as i64 / want as i64) as i32
        };
        left -= give;
        out.push(m + give);
    }
    out
}

fn row_heights(cells: &Cells, widths: &[i32]) -> Vec<i32> {
    let n = cells.iter().map(|(_, r, _)| r + 1).max().unwrap_or(0);
    let mut heights = vec![0; n];
    for (child, r, c) in cells {
        let (_, nat, _, _) = child.measure(gtk::Orientation::Vertical, widths[*c]);
        heights[*r] = heights[*r].max(nat);
    }
    heights
}

glib::wrapper! {
    pub struct Table(ObjectSubclass<imp::Table>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Table {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn attach(&self, child: &impl IsA<gtk::Widget>, row: usize, col: usize) {
        child.set_parent(self);
        self.imp().cells.borrow_mut().push((row, col));
    }
}

#[cfg(test)]
mod tests {
    use super::share;

    #[test]
    fn spare_width_is_shared_by_how_much_each_column_wants() {
        let (min, nat) = ([10, 10, 10], [10, 50, 110]);
        assert_eq!(share(&min, &nat, 200), [10, 50, 110]);
        assert_eq!(share(&min, &nat, 30), [10, 10, 10]);
        assert_eq!(share(&min, &nat, 10), [10, 10, 10]);
        let w = share(&min, &nat, 80);
        assert_eq!(w.iter().sum::<i32>(), 80);
        assert_eq!(w[0], 10);
        assert!(w[2] > w[1]);
    }
}
