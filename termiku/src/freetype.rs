use std::ffi::CString;

use ::freetype::freetype::*;

use crate::atlas::RectSize;

#[link(name = "freetype")]
extern {
    fn FT_Bitmap_Init(bitmap: *mut FT_Bitmap);
    fn FT_Bitmap_Copy(
        library: FT_Library,
        source: *const FT_Bitmap,
        destination: *mut FT_Bitmap,
    ) -> FT_Error;
    fn FT_Bitmap_Convert(
        library: FT_Library,
        source: *const FT_Bitmap,
        destination: *mut FT_Bitmap,
        alignment: FT_Int,
    ) -> FT_Error;
}



type FTResult<T> = Result<T, FT_Error>;

pub fn init_freetype() -> FTResult<FT_Library> {
    let mut freetype_lib: FT_Library = std::ptr::null_mut();
    let error = unsafe {
        FT_Init_FreeType(&mut freetype_lib)
    };
    
    if error != 0 {
        println!("Error loading freetype library, code: {}", error);
        Err(error)
    } else {
        Ok(freetype_lib)
    }
}

pub fn new_face(lib: FT_Library, path: &str) -> FTResult<FT_Face> {
    let mut face: FT_Face = std::ptr::null_mut();
    let path_c = CString::new(path).unwrap();
    
    let error = unsafe {
        FT_New_Face(
            lib,
            path_c.as_ptr(),
            0,
            &mut face
        )
    };
    
    if error != 0 {
        println!("Error loading freetype face, code: {}", error);
        Err(error)
    } else {
        Ok(face)
    }
}

pub fn set_char_size(face: FT_Face, size: i64) -> FTResult<()> {
    let error = unsafe {
        FT_Set_Char_Size(
            face,
            size * 64,
            size * 64,
            0,
            0
        )
    };
    
    if error != 0 {
        println!("Error setting freetype char size, code: {}", error);
        Err(error)
    } else {
        Ok(())
    }
}

pub fn render_glyph(_lib: FT_Library, face: FT_Face, glyph_index: u32) -> FTResult<FreeTypeGlyph> {
    let error = unsafe {
        FT_Load_Glyph(
            face,
            glyph_index,
            0
        )
    };
    
    if error != 0 {
        println!("Error loading glyph, code: {}", error);
        return Err(error);
    }
    
    let error = unsafe {
        FT_Render_Glyph(
            (*face).glyph,
            FT_Render_Mode_::FT_RENDER_MODE_NORMAL
        )
    };
    
    if error != 0 {
        println!("Error rendering glyph, code: {}", error);
        return Err(error);
    }
    
    // There's propably a memory leak somewhere
    
    // let mut uninit_bitmap = std::mem::MaybeUninit::<FT_Bitmap_>::uninit();
    // let bitmap_p = uninit_bitmap.as_mut_ptr();
    // let mut new_bitmap: FT_Bitmap = unsafe {
    //     FT_Bitmap_Init(bitmap_p);
    //     uninit_bitmap.assume_init()
    // };

    let glyph = unsafe {
        let glyph = *(*face).glyph;
        
        let metrics = glyph.metrics;
        
        let bitmap = glyph.bitmap;
        let buffer = std::slice::from_raw_parts(
            bitmap.buffer,
            (bitmap.pitch.abs() as u32 * bitmap.rows) as usize
        ).to_owned();

            
        FreeTypeGlyph {
            id: glyph_index,
            buffer,
            rows: bitmap.rows,
            pitch: bitmap.pitch.abs() as u32,
            width: metrics.width,
            height: metrics.height,
            bearing_x: metrics.horiBearingX,
            bearing_y: metrics.horiBearingY,
            advance_x: (*(*face).glyph).advance.x,
            advance_y: (*(*face).glyph).advance.y,
        }
    };
    
    Ok(glyph)
}

pub fn render_glyphs(lib: FT_Library, face: FT_Face, glyphs: &[u32]) -> FTResult<Vec<FreeTypeGlyph>> {
    let mut results: Vec<FreeTypeGlyph> = vec![];
    
    for &glyph in glyphs.iter() {
        let result = render_glyph(lib, face, glyph)?;
        results.push(result);
    }
    
    Ok(results)
}

#[derive(Debug, Clone)]
pub struct FreeTypeGlyph {
    id: u32,
    buffer: Vec<u8>,
    rows: u32,
    pitch: u32,
    pub width: i64,
    pub height: i64,
    pub bearing_x: i64,
    pub bearing_y: i64,
    advance_x: i64,
    advance_y: i64
}

impl FreeTypeGlyph {
    pub fn print(&self) {
        let iter = self.buffer.chunks(self.pitch as usize);
        for row in iter {
            for pixel in row {
                if *pixel < 200 {
                    print!(" ");
                } else {
                    print!("o");
                }
            }
            println!();
        }
    }
    
    pub fn size(&self) -> RectSize {
        RectSize {
            width: self.pitch,
            height: self.rows
        }
    }
    
    pub fn id(&self) -> u32 {
        self.id
    }
    
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }
}
