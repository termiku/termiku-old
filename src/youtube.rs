use libc::*;
use std::os::unix::io::AsRawFd;
use parking_lot::RawMutex;
use parking_lot::lock_api::RawMutex as _;

use crate::window::DEFAULT_BG;

pub const URL_PADDINGLESS_BASE64_RANGE: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Color(u8, u8, u8);

pub struct VlcVideoPlayer {
    pub pixel_buffer: Box<[Color]>,
    pub mutex: RawMutex,
    need_update: bool,
    width: u32,
    height: u32,
}

impl VlcVideoPlayer {
    pub fn get_frame(&mut self) -> Vec<u8> {
        self.mutex.lock();
        
        let buffer = unsafe {
            std::slice::from_raw_parts(self.pixel_buffer.as_ptr() as *mut Color as *mut u8, (self.width * self.height * 3) as usize)
        };
        
        self.mutex.unlock();
        
        buffer.to_vec()
    }
}

unsafe extern "C" fn lock_callback(object: *mut c_void, planes: *mut c_void) -> *mut c_void {
    let this = &mut *(object as *mut VlcVideoPlayer);
    
    this.mutex.lock();
    
    let planes_pointer: *mut *mut c_void = planes as *mut *mut c_void;
    
    *planes_pointer = this.pixel_buffer.as_mut().as_mut_ptr() as *mut c_void;
    
    std::ptr::null_mut()
}

unsafe extern "C" fn unlock_callback(object: *mut c_void, picture: *mut c_void, planes: *const *mut c_void) {
    let this = &mut *(object as *mut VlcVideoPlayer);
    
    this.need_update = true;
    
    this.mutex.unlock();
}

unsafe extern "C" fn display_callback(object: *mut c_void, picture: *mut c_void) {
    
}

pub struct YoutubeDlInstance {
    child: std::process::Child,
}

// caXgpo5Ezo4

impl YoutubeDlInstance {
    pub fn is_yt_dl_available() -> bool {
        let mut which = std::process::Command::new("which")
            .arg("youtube-dl")
            .spawn().unwrap();
        
        which.wait().unwrap().success()
    }
    
    pub fn new(id: &str) -> Self {
        let url = format!("https://www.youtube.com/watch?v={}", id);
        
        let youtube_dl = std::process::Command::new("youtube-dl")
            .stdout(std::process::Stdio::piped())
            .args(&["-v", "-o", "-", &url])
            .spawn().unwrap();
            
        Self {
            child: youtube_dl,
        }
    }
    
    fn get_stdout(&mut self) -> Option<&mut std::process::ChildStdout> {
        self.child.stdout.as_mut()
    }
    
    fn kill(&mut self) {
        self.child.kill().unwrap();
    }
}

pub struct YoutubeDlVlcInstance {
    ytdl: YoutubeDlInstance,
    instance: vlc::Instance,
    md: vlc::Media,
    mdp: vlc::MediaPlayer,
    pub player: Box<VlcVideoPlayer>
}

use std::sync::{Arc, Mutex};

pub struct WrappedYoutubeDlVlcInstance(pub Arc<Mutex<YoutubeDlVlcInstance>>);

impl WrappedYoutubeDlVlcInstance {
    pub fn new(ytdl: YoutubeDlInstance) -> Self {
        Self(Arc::new(Mutex::new(YoutubeDlVlcInstance::new(ytdl, 1280, 720))))
    }
}

unsafe impl Send for WrappedYoutubeDlVlcInstance {}
unsafe impl Sync for WrappedYoutubeDlVlcInstance {}

impl YoutubeDlVlcInstance<> {
    pub fn new(ytdl: YoutubeDlInstance, width: u32, height: u32) -> Self {
        let instance = vlc::Instance::new().unwrap();
        let md = vlc::Media::new_fd(&instance, ytdl.child.stdout.as_ref().unwrap().as_raw_fd()).unwrap();
        let mdp = vlc::MediaPlayer::new(&instance).unwrap();
        mdp.set_media(&md);

        let mdp_p = mdp.raw();
        
        let player = VlcVideoPlayer {
            pixel_buffer: vec![Color((DEFAULT_BG.0 * 255.0) as u8, (DEFAULT_BG.1 * 255.0) as u8, (DEFAULT_BG.2 * 255.0) as u8); (width * height) as usize].into_boxed_slice(),
            mutex: RawMutex::INIT,
            need_update: false,
            width,
            height
        };
        
        let player_box = Box::new(player);
        let player_p = Box::into_raw(player_box);
        let player_box = unsafe { Box::from_raw(player_p) };
        
        unsafe {
            vlc::sys::libvlc_video_set_callbacks(
                mdp_p,
                Some(lock_callback),
                Some(unlock_callback),
                Some(display_callback),
                player_p as *mut c_void,
            );
            
            
            vlc::sys::libvlc_video_set_format(
                mdp_p,
                std::ffi::CStr::from_bytes_with_nul(b"RV24\0").unwrap().as_ptr(),
                width,
                height,
                width * 3
            );
        }
        
        mdp.play().unwrap();
        
        Self {
            ytdl,
            player: player_box,
            instance,
            md,
            mdp,
            
        }
    }
    
    pub fn cleanup(&mut self) {
        self.mdp.stop();
        self.ytdl.kill();
    }
}