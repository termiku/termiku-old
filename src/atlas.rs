// mod atlas

// FIXME: This should be defined somewhere else.
type GlyphId = u32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RectSize {
    pub width:  u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub pos:  Point,
    pub size: RectSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Space {
    Empty,
    Filled(GlyphId),
}

#[derive(Clone, Copy, Debug)]
struct SpriteSlot {
    rect:  Rect,
    space: Space,
}

pub struct Atlas {
    size:   RectSize,
    slots:  Vec<SpriteSlot>,
}

impl Atlas {
    pub fn new(size: RectSize) -> Self {
        Atlas {
            size,
            slots: vec![],
            //pixels: vec![0; size.width * size.height]
        }
    }

    /// Tries to find an empty slot which can fit the specified rectangle.
    /// Returns a mutable reference to the SpriteSlot if one is found, and None otherwise.
    fn find_empty_slot(&mut self, size: RectSize) -> Option<&mut SpriteSlot> {
        self.slots.iter_mut()
                  .filter(|s| s.space == Space::Empty
                           && s.rect.size.width  >= size.width
                           && s.rect.size.height >= size.height)
                  .next()
    }

    /// Tries to find an open space for a new slot of the specified size.
    /// Returns a position where the new slot can be added, or None if no space is free.
    /// Not the most efficient algorithm in the world.
    fn find_open_slot_space(&self, size: RectSize) -> Option<Point> {
        if self.slots.is_empty() {
            return Some(Point { x: 0, y: 0 });
        } else {
            let slot  = self.slots.last().unwrap();

            let after_x = slot.rect.pos.x + slot.rect.size.width;
            let after_y = slot.rect.pos.y + slot.rect.size.height;

            // Check if we have enough space after the last slot
            if (after_x + size.width)  < self.size.width
            && (after_y + size.height) < self.size.height {
                return Some(Point { x: after_x, y: slot.rect.pos.y });
            // Let's try moving to a new line. We don't do any vertical packing and just use
            // The largest slot on the last line.
            } else {
                let slot_v = self.slots.iter()
                                       .max_by_key(|s| (s.rect.pos.y, s.rect.size.height))
                                       .unwrap();

                let after_y_v = slot_v.rect.pos.y + slot_v.rect.size.height;

                if (after_y_v + size.height) < self.size.height {
                    return Some(Point { x: 0, y: after_y_v });
                }
            }
        }

        return None;
    }

    /*
    /// Writes a vector of pixels to the Atlas according to the specified Rectangle.
    /// Assumes that `pixels` is large enough to fill the `rect`.
    fn write_pixels(&mut self, rect: &Rect, pixels: Vec<Pixel>) {
        // Convert (X, Y) coordinate to array index using x + y * w
        let start = rect.pos.x + rect.pos.y * self.size.width;

        for row in (0..rect.size.height) {
            // Offset start with the current row
            let start_r = start + row * self.size.width;
            let end_r   = start_r + rect.size.width;
            // Range to get one row of pixels.
            let start_p = row * rect.size.width;
            let end_p   = start_p + rect.size_width;
            self.pixels[start_r..end_r]
                .copy_from_slice(&pixels[start_p..end_p])
        }
    }
    */

    /// Tries inserting a Sprite with the specified GlyphId into the Atlas.
    /// Returns an Option containing the coordinates and size of the sprite in the atlas,
    /// or None if insertion failed.
    pub fn insert(&mut self, size: RectSize, gid: GlyphId) -> Option<Point> {
        // Check we don't already have this glyph.
        if self.slots.iter().any(|s| s.space == Space::Filled(gid)) {
            return None;
        }

        // If we can find a matching slot, simply reuse that.
        if let Some(slot) = self.find_empty_slot(size) {
            slot.rect.size = size;
            slot.space = Space::Filled(gid);
            // self.write_pixels(&slot.rect, sprite.pixels)
            return Some(slot.rect.pos);
        // Otherwise, we have to look for free space.
        } else if let Some(pos) = self.find_open_slot_space(size) {
            let slot = SpriteSlot {
                rect: Rect {
                    pos,
                    size
                },
                space: Space::Filled(gid)
            };

            self.slots.push(slot);
            return Some(pos);
        // No space
        } else {
            return None;
        }
    }

    /// Finds the slot associated with the glyph id.
    pub fn get(&self, gid: GlyphId) -> Option<Rect> {
        self.slots.iter()
                  .filter(|s| s.space == Space::Filled(gid))
                  .map(|s| s.rect)
                  .next()
    }

    // Complete shit code
    pub fn get_many(&self, gids: &[GlyphId]) -> Option<Vec<Rect>> {
        let v: Vec<_> = gids.iter()
                            .map(|&g| self.get(g))
                            .collect();

        if v.iter().all(|x| x.is_some()) {
            Some(v.iter().map(|x| x.unwrap()).collect())
        } else {
            None
        }
    }
}
