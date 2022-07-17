use thiserror::Error;

use crate::graphics::*;

#[derive(Error, Debug)]
pub enum BlendMapError {
    #[error("Source color {0} is out of range for this BlendMap")]
    InvalidSourceColor(u8),
}

/// A lookup table used by [`BlendMap`]s. This table stores destination color to blend color
/// mappings, where the indices are the destination colors and the values at those indices are the
/// blend colors.
pub type BlendMapping = [u8; 256];

/// A blend map containing a lookup table to match source colors with destination colors to
/// produce blended colors.
///
/// Some definitions:
/// * **source color**: colors in some source bitmap that is to be drawn onto a destination
/// * **destination color**: colors on the destination that will be drawn over by the source colors
/// * **blended color**: the final drawn color, found by looking up the source and destination colors
///
/// A blend map will not necessarily have mappings for all possible 256 source colors. But for each
/// source color, it will have 256 destination to blended color mappings.
#[derive(Clone)]
pub struct BlendMap {
    start_color: u8,
    end_color: u8,
    mapping: Box<[BlendMapping]>,
}

impl BlendMap {
    /// Creates and returns a new [`BlendMap`] with source color mappings for the given inclusive
    /// range only. The `start_color` and `end_color` may also be equal to create a blend map with
    /// only a single source color mapping.
    pub fn new(start_color: u8, end_color: u8) -> Self {
        let (start_color, end_color) = if start_color > end_color {
            (end_color, start_color)
        } else {
            (start_color, end_color)
        };
        let num_colors = (end_color - start_color) as usize + 1;
        BlendMap {
            start_color,
            end_color,
            mapping: vec![[0u8; 256]; num_colors].into_boxed_slice(),
        }
    }

    /// Creates and returns a new [`BlendMap`] with a single source color mapping which maps to
    /// a table pre-calculated for the given palette based on the color gradient specified. The
    /// resulting blend map can be used to create simple colored translucency overlay effects. The
    /// starting color in the gradient is used as the source color mapping in the returned blend
    /// map.
    pub fn new_translucency_map(gradient_start: u8, gradient_end: u8, palette: &Palette) -> Self {
        let (gradient_start, gradient_end) = if gradient_start > gradient_end {
            (gradient_end, gradient_start)
        } else {
            (gradient_start, gradient_end)
        };
        let gradient_size = gradient_end - gradient_start + 1;
        let source_color = gradient_start;

        let mut blend_map = Self::new(source_color, source_color);
        for idx in 0..=255 {
            let (r, g, b) = from_rgb32(palette[idx]);
            let lit = (luminance(r, g, b) * 255.0) as u8;
            blend_map.set_mapping(
                source_color,
                idx as u8,
                (gradient_size - 1) - (lit / (256 / gradient_size as u32) as u8) + source_color
            ).unwrap();
        }
        blend_map
    }

    /// The beginning source color that is mapped in this blend map.
    #[inline]
    pub fn start_color(&self) -> u8 {
        self.start_color
    }

    /// The ending source color that is mapped in this blend map.
    #[inline]
    pub fn end_color(&self) -> u8 {
        self.end_color
    }

    /// Returns true if the given source color is mapped in this blend map.
    #[inline]
    pub fn is_mapped(&self, color: u8) -> bool {
        color >= self.start_color && color <= self.end_color
    }

    #[inline]
    fn get_mapping_index(&self, color: u8) -> Option<usize> {
        if color >= self.start_color && color <= self.end_color {
            let index = (color - self.start_color) as usize;
            Some(index)
        } else {
            None
        }
    }

    /// Returns a reference to the destination-to-blend color mapping table for the given source
    /// color. Returns `None` if the specified source color is not in this blend map.
    #[inline]
    pub fn get_mapping(&self, color: u8) -> Option<&BlendMapping> {
        if let Some(index) = self.get_mapping_index(color) {
            // safety: index cannot be outside 0-255 since color and start_color are both u8
            unsafe { Some(self.mapping.get_unchecked(index)) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the destination-to-blend color mapping table for the given
    /// source color. Returns `None` if the specified source color is not in this blend map.
    #[inline]
    pub fn get_mapping_mut(&mut self, color: u8) -> Option<&mut BlendMapping> {
        if let Some(index) = self.get_mapping_index(color) {
            // safety: index cannot be outside 0-255 since color and start_color are both u8
            unsafe { Some(self.mapping.get_unchecked_mut(index)) }
        } else {
            None
        }
    }

    /// Sets the blend color mapping for the given source color and destination color combination.
    pub fn set_mapping(&mut self, source_color: u8, dest_color: u8, blended_color: u8) -> Result<(), BlendMapError> {
        if let Some(mapping) = self.get_mapping_mut(source_color) {
            mapping[dest_color as usize] = blended_color;
            Ok(())
        } else {
            Err(BlendMapError::InvalidSourceColor(source_color))
        }
    }


    /// Sets a series of blend color mappings for the given source color and starting from a base
    /// destination color.
    pub fn set_mappings<const N: usize>(&mut self, source_color: u8, base_dest_color: u8, mappings: [u8; N]) -> Result<(), BlendMapError> {
        if let Some(mapping) = self.get_mapping_mut(source_color) {
            assert!((base_dest_color as usize + N - 1) <= 255, "mappings array is too big for the remaining colors available");
            for index in 0..N {
                mapping[index + base_dest_color as usize] = mappings[index];
            }
            Ok(())
        } else {
            Err(BlendMapError::InvalidSourceColor(source_color))
        }
    }

    /// Returns the blend color for the given source and destination colors. If the source color
    /// is not in this blend map, `None` is returned.
    #[inline]
    pub fn blend(&self, source_color: u8, dest_color: u8) -> Option<u8> {
        if let Some(mapping) = self.get_mapping(source_color) {
            Some(mapping[dest_color as usize])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use claim::*;

    use super::*;

    #[test]
    pub fn create() -> Result<(), BlendMapError> {
        let blend_map = BlendMap::new(10, 12);
        assert_eq!(10, blend_map.start_color());
        assert_eq!(12, blend_map.end_color());
        assert!(blend_map.is_mapped(10));
        assert!(blend_map.is_mapped(11));
        assert!(blend_map.is_mapped(12));
        assert!(!blend_map.is_mapped(9));
        assert!(!blend_map.is_mapped(13));
        assert_some!(blend_map.get_mapping(10));
        assert_some!(blend_map.get_mapping(11));
        assert_some!(blend_map.get_mapping(12));
        assert_none!(blend_map.get_mapping(9));
        assert_none!(blend_map.get_mapping(13));

        let blend_map = BlendMap::new(12, 10);
        assert_eq!(10, blend_map.start_color());
        assert_eq!(12, blend_map.end_color());
        assert!(blend_map.is_mapped(10));
        assert!(blend_map.is_mapped(11));
        assert!(blend_map.is_mapped(12));
        assert!(!blend_map.is_mapped(9));
        assert!(!blend_map.is_mapped(13));
        assert_some!(blend_map.get_mapping(10));
        assert_some!(blend_map.get_mapping(11));
        assert_some!(blend_map.get_mapping(12));
        assert_none!(blend_map.get_mapping(9));
        assert_none!(blend_map.get_mapping(13));

        let blend_map = BlendMap::new(130, 130);
        assert_eq!(130, blend_map.start_color());
        assert_eq!(130, blend_map.end_color());
        assert!(blend_map.is_mapped(130));
        assert!(!blend_map.is_mapped(129));
        assert!(!blend_map.is_mapped(131));
        assert_some!(blend_map.get_mapping(130));
        assert_none!(blend_map.get_mapping(129));
        assert_none!(blend_map.get_mapping(131));

        Ok(())
    }

    #[test]
    pub fn mapping() -> Result<(), BlendMapError> {
        let mut blend_map = BlendMap::new(16, 31);

        assert_none!(blend_map.blend(15, 0));
        assert_eq!(Some(0), blend_map.blend(16, 0));
        assert_eq!(Some(0), blend_map.blend(16, 1));
        assert_ok!(blend_map.set_mapping(16, 0, 116));
        assert_eq!(Some(116), blend_map.blend(16, 0));
        assert_eq!(Some(0), blend_map.blend(16, 1));

        let mapping = blend_map.get_mapping(16).unwrap();
        assert_eq!(116, mapping[0]);
        assert_eq!(0, mapping[1]);

        assert_eq!(Some(0), blend_map.blend(17, 0));
        assert_ok!(blend_map.set_mapping(17, 0, 117));
        assert_eq!(Some(117), blend_map.blend(17, 0));
        let mapping = blend_map.get_mapping_mut(17).unwrap();
        assert_eq!(117, mapping[0]);
        mapping[0] = 217;
        assert_eq!(Some(217), blend_map.blend(17, 0));

        assert_matches!(
            blend_map.set_mapping(64, 1, 2),
            Err(BlendMapError::InvalidSourceColor(64))
        );

        Ok(())
    }

    #[test]
    pub fn bulk_mappings() -> Result<(), BlendMapError> {
        let mut blend_map = BlendMap::new(0, 7);

        let mapping = blend_map.get_mapping(2).unwrap();
        assert_eq!([0, 0, 0, 0, 0, 0, 0, 0], mapping[0..8]);

        assert_ok!(blend_map.set_mappings(2, 4, [1, 2, 3, 4, 5, 6, 7, 8]));

        let mapping = blend_map.get_mapping(2).unwrap();
        assert_eq!(
            [0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0],
            mapping[0..16]
        );


        Ok(())
    }
}