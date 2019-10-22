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

pub fn create_harfbuzz_buffer(data: &str) -> Buffer {
    let mut buffer = Buffer::with(data);
    buffer.set_direction(Direction::LTR);
    buffer.set_script(HB_SCRIPT_LATIN);
    buffer.set_language(Language::from_string("en"));
    buffer
}

pub unsafe fn harfbuzz_shape(font: *mut hb_font_t, buffer: *mut hb_buffer_t) {
    hb_shape(font, buffer, std::ptr::null(), 0);
}

fn blob_from_file(path: &str) -> Blob {
    let data = std::fs::read(path).unwrap();
    let blob = Blob::new_from_arc_vec(Arc::new(data));
    if blob.is_empty() {
        panic!("File blob is empty for some reason");
    }
    blob
}
