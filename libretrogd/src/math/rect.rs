/// Represents a 2D rectangle, using integer coordinates and dimensions.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    #[inline]
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a new rect from the specified coordinates. Automatically determines if the
    /// coordinates provided are swapped (where the right/bottom coordinate is provided before the
    /// left/top). All of the coordinates are inclusive.
    ///
    /// # Arguments
    ///
    /// * `left`: the left x coordinate
    /// * `top`: the top y coordinate
    /// * `right`: the right x coordinate
    /// * `bottom`: the bottom y coordinate
    pub fn from_coords(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        let x;
        let y;
        let width;
        let height;

        if left <= right {
            x = left;
            width = (right - left).abs() + 1;
        } else {
            x = right;
            width = (left - right).abs() + 1;
        }

        if top <= bottom {
            y = top;
            height = (bottom - top).abs() + 1;
        } else {
            y = bottom;
            height = (top - bottom).abs() + 1;
        }

        Rect {
            x,
            y,
            width: width as u32,
            height: height as u32,
        }
    }

    pub fn set_from_coords(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        if left <= right {
            self.x = left;
            self.width = ((right - left).abs() + 1) as u32;
        } else {
            self.x = right;
            self.width = ((left - right).abs() + 1) as u32;
        }

        if top <= bottom {
            self.y = top;
            self.height = ((bottom - top).abs() + 1) as u32;
        } else {
            self.y = bottom;
            self.height = ((top - bottom).abs() + 1) as u32;
        }
    }

    /// Calculates the right-most x coordinate contained by this rect.
    #[inline]
    pub fn right(&self) -> i32 {
        if self.width > 0 {
            self.x + self.width as i32 - 1
        } else {
            self.x
        }
    }

    /// Calculates the bottom-most y coordinate contained by this rect.
    #[inline]
    pub fn bottom(&self) -> i32 {
        if self.height > 0 {
            self.y + self.height as i32 - 1
        } else {
            self.y
        }
    }

    /// Returns true if the given point is contained within the bounds of this rect.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        (self.x <= x) && (self.right() >= x) && (self.y <= y) && (self.bottom() >= y)
    }

    /// Returns true if the given rect is contained completely within the bounds of this rect.
    pub fn contains_rect(&self, other: &Rect) -> bool {
        (other.x >= self.x && other.x < self.right())
            && (other.right() > self.x && other.right() <= self.right())
            && (other.y >= self.y && other.y < self.bottom())
            && (other.bottom() > self.y && other.bottom() <= self.bottom())
    }

    /// Returns true if the given rect at least partially overlaps the bounds of this rect.
    pub fn overlaps(&self, other: &Rect) -> bool {
        (self.x <= other.right())
            && (self.right() >= other.x)
            && (self.y <= other.bottom())
            && (self.bottom() >= other.y)
    }

    pub fn clamp_to(&mut self, other: &Rect) -> bool {
        if !self.overlaps(other) {
            // not possible to clamp this rect to the other rect as they do not overlap at all
            false
        } else {
            // the rects at least partially overlap, so we will clamp this rect to the overlapping
            // region of the other rect
            let mut x1 = self.x;
            let mut y1 = self.y;
            let mut x2 = self.right();
            let mut y2 = self.bottom();
            if y1 < other.y {
                y1 = other.y;
            }
            if y1 > other.bottom() {
                y1 = other.bottom();
            }
            if y2 < other.y {
                y2 = other.y;
            }
            if y2 > other.bottom() {
                y2 = other.bottom();
            }
            if x1 < other.x {
                x1 = other.x;
            }
            if x1 > other.right() {
                x1 = other.right();
            }
            if x2 < other.x {
                x2 = other.x;
            }
            if x2 > other.right() {
                x2 = other.right();
            }

            self.set_from_coords(x1, y1, x2, y2);
            true
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn right_and_left() {
        let rect = Rect {
            x: 5,
            y: 6,
            width: 16,
            height: 12,
        };
        assert_eq!(20, rect.right());
        assert_eq!(17, rect.bottom());

        let rect = Rect {
            x: -11,
            y: -25,
            width: 16,
            height: 12,
        };
        assert_eq!(4, rect.right());
        assert_eq!(-14, rect.bottom());
    }

    #[test]
    pub fn create_from_coords() {
        let rect = Rect::from_coords(10, 15, 20, 30);
        assert_eq!(10, rect.x);
        assert_eq!(15, rect.y);
        assert_eq!(11, rect.width);
        assert_eq!(16, rect.height);
        assert_eq!(20, rect.right());
        assert_eq!(30, rect.bottom());

        let rect = Rect::from_coords(-5, -13, 6, -2);
        assert_eq!(-5, rect.x);
        assert_eq!(-13, rect.y);
        assert_eq!(12, rect.width);
        assert_eq!(12, rect.height);
        assert_eq!(6, rect.right());
        assert_eq!(-2, rect.bottom());
    }

    #[test]
    pub fn create_from_coords_with_swapped_order() {
        let rect = Rect::from_coords(20, 30, 10, 15);
        assert_eq!(10, rect.x);
        assert_eq!(15, rect.y);
        assert_eq!(11, rect.width);
        assert_eq!(16, rect.height);
        assert_eq!(20, rect.right());
        assert_eq!(30, rect.bottom());

        let rect = Rect::from_coords(6, -2, -5, -13);
        assert_eq!(-5, rect.x);
        assert_eq!(-13, rect.y);
        assert_eq!(12, rect.width);
        assert_eq!(12, rect.height);
        assert_eq!(6, rect.right());
        assert_eq!(-2, rect.bottom());
    }

    #[test]
    pub fn test_contains_point() {
        let r = Rect::from_coords(10, 10, 20, 20);

        assert!(r.contains_point(10, 10));
        assert!(r.contains_point(15, 15));
        assert!(r.contains_point(20, 20));

        assert!(!r.contains_point(12, 30));
        assert!(!r.contains_point(8, 12));
        assert!(!r.contains_point(25, 16));
        assert!(!r.contains_point(17, 4));
    }

    #[test]
    pub fn test_contains_rect() {
        let r = Rect::from_coords(10, 10, 20, 20);

        assert!(r.contains_rect(&Rect::from_coords(12, 12, 15, 15)));
        assert!(r.contains_rect(&Rect::from_coords(10, 10, 15, 15)));
        assert!(r.contains_rect(&Rect::from_coords(15, 15, 20, 20)));
        assert!(r.contains_rect(&Rect::from_coords(10, 12, 20, 15)));
        assert!(r.contains_rect(&Rect::from_coords(12, 10, 15, 20)));

        assert!(!r.contains_rect(&Rect::from_coords(5, 5, 15, 15)));
        assert!(!r.contains_rect(&Rect::from_coords(15, 15, 25, 25)));

        assert!(!r.contains_rect(&Rect::from_coords(2, 2, 8, 4)));
        assert!(!r.contains_rect(&Rect::from_coords(12, 21, 18, 25)));
        assert!(!r.contains_rect(&Rect::from_coords(22, 12, 32, 17)));
    }

    #[test]
    pub fn test_overlaps() {
        let r = Rect::from_coords(10, 10, 20, 20);

        assert!(r.overlaps(&Rect::from_coords(12, 12, 15, 15)));
        assert!(r.overlaps(&Rect::from_coords(10, 10, 15, 15)));
        assert!(r.overlaps(&Rect::from_coords(15, 15, 20, 20)));
        assert!(r.overlaps(&Rect::from_coords(10, 12, 20, 15)));
        assert!(r.overlaps(&Rect::from_coords(12, 10, 15, 20)));

        assert!(r.overlaps(&Rect::from_coords(12, 5, 18, 10)));
        assert!(r.overlaps(&Rect::from_coords(13, 20, 16, 25)));
        assert!(r.overlaps(&Rect::from_coords(5, 12, 10, 18)));
        assert!(r.overlaps(&Rect::from_coords(20, 13, 25, 16)));

        assert!(r.overlaps(&Rect::from_coords(5, 5, 15, 15)));
        assert!(r.overlaps(&Rect::from_coords(15, 15, 25, 25)));

        assert!(!r.overlaps(&Rect::from_coords(2, 2, 8, 4)));
        assert!(!r.overlaps(&Rect::from_coords(12, 21, 18, 25)));
        assert!(!r.overlaps(&Rect::from_coords(22, 12, 32, 17)));

        assert!(!r.overlaps(&Rect::from_coords(12, 5, 18, 9)));
        assert!(!r.overlaps(&Rect::from_coords(13, 21, 16, 25)));
        assert!(!r.overlaps(&Rect::from_coords(5, 12, 9, 18)));
        assert!(!r.overlaps(&Rect::from_coords(21, 13, 25, 16)));
    }
}
