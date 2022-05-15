use crate::{Bitmap, Rect};

pub enum BlitMethod {
    Solid,
    Transparent(u8),
}

/// Clips the region for a source bitmap to be used in a subsequent blit operation. The source
/// region will be clipped against the clipping region given for the destination bitmap. The
/// top-left coordinates of the location to blit to on the destination bitmap are also adjusted
/// only if necessary based on the clipping performed.
///
/// # Arguments
///
/// * `dest_clip_region`: the clipping region for the destination bitmap
/// * `src_blit_region`: the region on the source bitmap that is to be blitted, which may be
///   clipped if necessary to at least partially fit into the destination clipping region given
/// * `dest_x`: the x (left) coordinate of the location on the destination bitmap to blit the
///   source to, which may be adjusted as necessary during clipping
/// * `dest_y`: the y (top) coordinate of the location on the destination bitmap to blit the source
///   to, which may be adjusted as necessary during clipping
///
/// returns: true if the results of the clip is partially or entirely visible on the destination
/// bitmap, or false if the blit is entirely outside of the destination bitmap (and so no blit
/// needs to occur)
pub fn clip_blit(
    dest_clip_region: &Rect,
    src_blit_region: &mut Rect,
    dest_x: &mut i32,
    dest_y: &mut i32,
) -> bool {
    // off the left edge?
    if *dest_x < dest_clip_region.x {
        // completely off the left edge?
        if (*dest_x + src_blit_region.width as i32 - 1) < dest_clip_region.x {
            return false;
        }

        let offset = dest_clip_region.x - *dest_x;
        src_blit_region.x += offset;
        src_blit_region.width = (src_blit_region.width as i32 - offset) as u32;
        *dest_x = dest_clip_region.x;
    }

    // off the right edge?
    if *dest_x > dest_clip_region.width as i32 - src_blit_region.width as i32 {
        // completely off the right edge?
        if *dest_x > dest_clip_region.right() {
            return false;
        }

        let offset = *dest_x + src_blit_region.width as i32 - dest_clip_region.width as i32;
        src_blit_region.width = (src_blit_region.width as i32 - offset) as u32;
    }

    // off the top edge?
    if *dest_y < dest_clip_region.y {
        // completely off the top edge?
        if (*dest_y + src_blit_region.height as i32 - 1) < dest_clip_region.y {
            return false;
        }

        let offset = dest_clip_region.y - *dest_y;
        src_blit_region.y += offset;
        src_blit_region.height = (src_blit_region.height as i32 - offset) as u32;
        *dest_y = dest_clip_region.y;
    }

    // off the bottom edge?
    if *dest_y > dest_clip_region.height as i32 - src_blit_region.height as i32 {
        // completely off the bottom edge?
        if *dest_y > dest_clip_region.bottom() {
            return false;
        }

        let offset = *dest_y + src_blit_region.height as i32 - dest_clip_region.height as i32;
        src_blit_region.height = (src_blit_region.height as i32 - offset) as u32;
    }

    true
}

impl Bitmap {
    pub unsafe fn solid_blit(&mut self, src: &Bitmap, src_region: &Rect, dest_x: i32, dest_y: i32) {
        let src_row_length = src_region.width as usize;
        let src_pitch = src.width as usize;
        let dest_pitch = self.width as usize;
        let mut src_pixels = src.pixels_at_ptr_unchecked(src_region.x, src_region.y);
        let mut dest_pixels = self.pixels_at_mut_ptr_unchecked(dest_x, dest_y);

        for _ in 0..src_region.height {
            dest_pixels.copy_from(src_pixels, src_row_length);
            src_pixels = src_pixels.add(src_pitch);
            dest_pixels = dest_pixels.add(dest_pitch);
        }
    }

    pub unsafe fn transparent_blit(
        &mut self,
        src: &Bitmap,
        src_region: &Rect,
        dest_x: i32,
        dest_y: i32,
        transparent_color: u8,
    ) {
        let src_next_row_inc = (src.width - src_region.width) as usize;
        let dest_next_row_inc = (self.width - src_region.width) as usize;
        let mut src_pixels = src.pixels_at_ptr_unchecked(src_region.x, src_region.y);
        let mut dest_pixels = self.pixels_at_mut_ptr_unchecked(dest_x, dest_y);

        for _ in 0..src_region.height {
            for _ in 0..src_region.width {
                let pixel = *src_pixels;
                if pixel != transparent_color {
                    *dest_pixels = pixel;
                }

                src_pixels = src_pixels.add(1);
                dest_pixels = dest_pixels.add(1);
            }

            src_pixels = src_pixels.add(src_next_row_inc);
            dest_pixels = dest_pixels.add(dest_next_row_inc);
        }
    }

    pub fn blit_region(
        &mut self,
        method: BlitMethod,
        src: &Bitmap,
        src_region: &Rect,
        mut dest_x: i32,
        mut dest_y: i32,
    ) {
        let mut src_region = *src_region;
        if !clip_blit(
            self.clip_region(),
            &mut src_region,
            &mut dest_x,
            &mut dest_y,
        ) {
            return;
        }

        unsafe {
            self.blit_region_unchecked(method, src, &src_region, dest_x, dest_y);
        };
    }

    #[inline]
    pub unsafe fn blit_region_unchecked(
        &mut self,
        method: BlitMethod,
        src: &Bitmap,
        src_region: &Rect,
        dest_x: i32,
        dest_y: i32,
    ) {
        use BlitMethod::*;
        match method {
            Solid => self.solid_blit(src, src_region, dest_x, dest_y),
            Transparent(transparent_color) => {
                self.transparent_blit(src, src_region, dest_x, dest_y, transparent_color)
            }
        }
    }

    #[inline]
    pub fn blit(&mut self, method: BlitMethod, src: &Bitmap, x: i32, y: i32) {
        let src_region = Rect::new(0, 0, src.width, src.height);
        self.blit_region(method, src, &src_region, x, y);
    }

    #[inline]
    pub unsafe fn blit_unchecked(&mut self, method: BlitMethod, src: &Bitmap, x: i32, y: i32) {
        let src_region = Rect::new(0, 0, src.width, src.height);
        self.blit_region_unchecked(method, src, &src_region, x, y);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn clip_blit_regions() {
        let dest = Rect::new(0, 0, 320, 240);

        let mut src: Rect;
        let mut x: i32;
        let mut y: i32;

        src = Rect::new(0, 0, 16, 16);
        x = 10;
        y = 10;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 0, 16, 16));
        assert_eq!(10, x);
        assert_eq!(10, y);

        // left edge

        src = Rect::new(0, 0, 16, 16);
        x = 0;
        y = 10;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 0, 16, 16));
        assert_eq!(0, x);
        assert_eq!(10, y);

        src = Rect::new(0, 0, 16, 16);
        x = -5;
        y = 10;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(5, 0, 11, 16));
        assert_eq!(0, x);
        assert_eq!(10, y);

        src = Rect::new(0, 0, 16, 16);
        x = -16;
        y = 10;
        assert!(!clip_blit(&dest, &mut src, &mut x, &mut y));

        // right edge

        src = Rect::new(0, 0, 16, 16);
        x = 304;
        y = 10;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 0, 16, 16));
        assert_eq!(304, x);
        assert_eq!(10, y);

        src = Rect::new(0, 0, 16, 16);
        x = 310;
        y = 10;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 0, 10, 16));
        assert_eq!(310, x);
        assert_eq!(10, y);

        src = Rect::new(0, 0, 16, 16);
        x = 320;
        y = 10;
        assert!(!clip_blit(&dest, &mut src, &mut x, &mut y));

        // top edge

        src = Rect::new(0, 0, 16, 16);
        x = 10;
        y = 0;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 0, 16, 16));
        assert_eq!(10, x);
        assert_eq!(0, y);

        src = Rect::new(0, 0, 16, 16);
        x = 10;
        y = -5;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 5, 16, 11));
        assert_eq!(10, x);
        assert_eq!(0, y);

        src = Rect::new(0, 0, 16, 16);
        x = 10;
        y = -16;
        assert!(!clip_blit(&dest, &mut src, &mut x, &mut y));

        // bottom edge

        src = Rect::new(0, 0, 16, 16);
        x = 10;
        y = 224;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 0, 16, 16));
        assert_eq!(10, x);
        assert_eq!(224, y);

        src = Rect::new(0, 0, 16, 16);
        x = 10;
        y = 229;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 0, 16, 11));
        assert_eq!(10, x);
        assert_eq!(229, y);

        src = Rect::new(0, 0, 16, 16);
        x = 10;
        y = 240;
        assert!(!clip_blit(&dest, &mut src, &mut x, &mut y));

        src = Rect::new(16, 16, 16, 16);
        x = -1;
        y = 112;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(17, 16, 15, 16));
        assert_eq!(0, x);
        assert_eq!(112, y);
    }

    #[test]
    pub fn clip_blit_regions_large_source() {
        let dest = Rect::new(0, 0, 64, 64);

        let mut src: Rect;
        let mut x: i32;
        let mut y: i32;

        src = Rect::new(0, 0, 128, 128);
        x = 0;
        y = 0;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 0, 64, 64));
        assert_eq!(0, x);
        assert_eq!(0, y);

        src = Rect::new(0, 0, 128, 128);
        x = -16;
        y = -24;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(16, 24, 64, 64));
        assert_eq!(0, x);
        assert_eq!(0, y);

        src = Rect::new(0, 0, 32, 128);
        x = 10;
        y = -20;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(0, 20, 32, 64));
        assert_eq!(10, x);
        assert_eq!(0, y);

        src = Rect::new(0, 0, 128, 32);
        x = -20;
        y = 10;
        assert!(clip_blit(&dest, &mut src, &mut x, &mut y));
        assert_eq!(src, Rect::new(20, 0, 64, 32));
        assert_eq!(0, x);
        assert_eq!(10, y);
    }
}
