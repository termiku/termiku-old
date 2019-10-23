use ::freetype::freetype::*;
use std::ffi::CString;
use harfbuzz::sys::*;
use harfbuzz::*;

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

pub fn set_char_size(face: FT_Face) -> FTResult<()> {
    let error = unsafe {
        FT_Set_Char_Size(
            face,
            3000,
            3000,
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

pub fn render_glyph(face: FT_Face, glyph_index: u32) -> FTResult<FreeTypeGlyph> {
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
    
    let glyph = unsafe {
        let bitmap = (*(*face).glyph).bitmap;
            let buffer = std::slice::from_raw_parts(
                bitmap.buffer,
                (bitmap.pitch.abs() as u32 * bitmap.rows) as usize
            ).to_owned();
            
            FreeTypeGlyph {
                buffer,
                rows: bitmap.rows,
                pitch: bitmap.pitch.abs() as u32,
                advance_x: (*(*face).glyph).advance.x,
                advance_y: (*(*face).glyph).advance.y,
            }
        };
    
    Ok(glyph)
}

#[derive(Debug)]
pub struct FreeTypeGlyph {
    buffer: Vec<u8>,
    rows: u32,
    pitch: u32,
    advance_x: i64,
    advance_y: i64
}

impl FreeTypeGlyph {
    pub fn print(&self) {
        let mut iter = self.buffer.chunks(self.pitch as usize);
        while let Some(row) = iter.next() {
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
}
