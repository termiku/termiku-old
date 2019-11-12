use std::borrow::Cow;

use glium::Display;

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

impl Rect {
    // min
    pub fn top_left(&self) -> Point {
        self.pos
    }
    
    pub fn top_right(&self) -> Point {
        let Point { x, y } = self.pos;
        Point {
            x: x + self.size.width,
            y
        }        
    }
    
    // max
    pub fn bottom_right(&self) -> Point {
        let Point { x, y } = self.pos;
        Point {
            x,
            y: y + self.size.height
        }  
    }
    
    pub fn bottom_left(&self) -> Point {
        let Point { x, y } = self.pos;
        Point {
            x: x + self.size.width,
            y: y + self.size.height
        }  
    }
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
    pub atlas: glium::texture::texture2d::Texture2d,
    size:   RectSize,
    slots:  Vec<SpriteSlot>,
}

impl Atlas {
    pub fn new(display: &Display, size: RectSize) -> Self {
        Atlas {
            atlas: glium::texture::Texture2d::with_format(
                display,
                glium::texture::RawImage2d {
                    data: Cow::Owned(vec![0u8; size.width as usize * size.height as usize]),
                    width: size.width,
                    height: size.height,
                    format: glium::texture::ClientFormat::U8,
                },
                glium::texture::UncompressedFloatFormat::U8,
                glium::texture::MipmapsOption::NoMipmap,
            ).unwrap(),
            size,
            slots: vec![],
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
            if (after_x + size.width)  <= self.size.width
            && (after_y + size.height) <= self.size.height {
                return Some(Point { x: after_x, y: slot.rect.pos.y });
            // Let's try moving to a new line. We don't do any vertical packing and just use
            // The largest slot on the last line.
            } else {
                let slot_v = self.slots.iter()
                                       .max_by_key(|s| (s.rect.pos.y, s.rect.size.height))
                                       .unwrap();

                let after_y_v = slot_v.rect.pos.y + slot_v.rect.size.height;

                if (after_y_v + size.height) <= self.size.height {
                    return Some(Point { x: 0, y: after_y_v });
                }
            }
        }

        return None;
    }
    
    fn write_to_texture(&mut self, rect: Rect, data: &[u8]) {
        self.atlas.main_level().write(
                    glium::Rect {
                        left: rect.top_left().x,
                        bottom: rect.top_left().y,
                        width: rect.size.width,
                        height: rect.size.height,
                    },
                    glium::texture::RawImage2d {
                        data: Cow::Borrowed(data),
                        width: rect.size.width,
                        height: rect.size.height,
                        format: glium::texture::ClientFormat::U8,
                    },
                );
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
    pub fn insert(&mut self, size: RectSize, gid: GlyphId, data: &[u8]) -> Option<Point> {
        // Check we don't already have this glyph.
        if let Some(rect) = self.get(gid) {
            return Some(rect.pos);
        }
        
        let mut to_be_returned: Option<Point> = None;
        let mut rect_to_update: Option<Rect> = None;

        // If we can find a matching slot, simply reuse that.
        if let Some(slot) = self.find_empty_slot(size) {
            slot.rect.size = size;
            slot.space = Space::Filled(gid);
            
            rect_to_update = Some(slot.rect);
            to_be_returned = Some(slot.rect.pos);
        // Otherwise, we have to look for free space.
        } else if let Some(pos) = self.find_open_slot_space(size) {
            let rect = Rect {
                pos,
                size
            };
            let slot = SpriteSlot {
                rect,
                space: Space::Filled(gid)
            };
            
            self.slots.push(slot);
            
            rect_to_update = Some(rect);
            to_be_returned = Some(pos);

        }
        // else, no space
        
        if let Some(rect) = rect_to_update {
            self.write_to_texture(rect, data);
        }
        
        to_be_returned
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
