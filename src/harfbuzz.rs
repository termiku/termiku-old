use harfbuzz::sys::*;
use std::sync::Arc;
use harfbuzz::*;

type HBResult<T> = Result<T, ()>;

pub fn create_harfbuzz_font(path: &str) -> HBResult<*mut hb_font_t> {
    let blob = blob_from_file(path);
    let blob_p = blob.as_raw();
    
    let face_p = unsafe {
        hb_face_create(blob_p, 0)
    };
    
    let font_p = unsafe {
        hb_font_create(face_p)
    };
    
    if font_p.is_null() {
        println!("Error creating face");
        std::process::exit(1);
    }
    
    unsafe {
        hb_font_set_scale(font_p, 1000, 1000);
    }
    
    Ok(font_p)
}

pub fn create_harfbuzz_buffer(capacity: usize) -> Buffer {
    let mut buffer = Buffer::with_capacity(capacity);
    buffer.set_direction(Direction::LTR);
    buffer.set_script(HB_SCRIPT_LATIN);
    buffer.set_language(Language::from_string("en"));
    buffer
}

pub unsafe fn add_slice_to_buffer(buffer: *mut hb_buffer_t, data: &[u8]) {
    hb_buffer_add_utf8(
        buffer,
        data.as_ptr() as *const std::os::raw::c_char,
        data.len() as std::os::raw::c_int,
        0,
        data.len() as std::os::raw::c_int,
    );
}

pub unsafe fn harfbuzz_shape(font: *mut hb_font_t, buffer: *mut hb_buffer_t) {
    hb_shape(font, buffer, std::ptr::null(), 0);
}

pub unsafe fn print_harfbuzz_buffer_info(font: *mut hb_font_t, buffer: *mut hb_buffer_t) {
    let buffer_length: u32 = hb_buffer_get_length(buffer);
    let glyph_infos_p = hb_buffer_get_glyph_infos(buffer, std::ptr::null_mut());
    let glyph_infos = std::slice::from_raw_parts(glyph_infos_p, buffer_length as usize);
    
    
    for info in glyph_infos {
        let gid = info.codepoint;
        let mut name_buffer = [0i8; 32];
        let name_buffer_p = name_buffer.as_mut_ptr();
        
        hb_font_get_glyph_name(font, gid, name_buffer_p, 32);
        // println!("gid: {}, name: {}", gid, String::from_utf8(name_buffer.iter().map(|&c| c as u8).collect()).unwrap())
    }
}

pub unsafe fn get_buffer_glyph(buffer: *mut hb_buffer_t) -> Vec<u32> {
    let buffer_length: u32 = hb_buffer_get_length(buffer);
    let glyph_infos_p = hb_buffer_get_glyph_infos(buffer, std::ptr::null_mut());
    let glyph_infos = std::slice::from_raw_parts(glyph_infos_p, buffer_length as usize);
    
    glyph_infos.iter().map(|i| i.codepoint).collect()
}

fn blob_from_file(path: &str) -> Blob {
    let data = std::fs::read(path).unwrap();
    let blob = Blob::new_from_arc_vec(Arc::new(data));
    if blob.is_empty() {
        panic!("File blob is empty for some reason");
    }
    blob
}
