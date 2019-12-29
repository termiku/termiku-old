#[derive(Clone, Debug)]
pub struct ScreenEvent {
    pub terminal_id: usize,
    pub event: ScreenEventType
}

#[derive(Clone, Debug)]
pub enum ScreenEventType {
    PlayYoutubeVideo(String)
}

