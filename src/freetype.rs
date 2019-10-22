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
            1000,
            1000,
            0,
            0
        )
    };
    
    if error != 0 {
        println!("Error loading freetype face, code: {}", error);
        Err(error)
    } else {
        Ok(())
    }
}
