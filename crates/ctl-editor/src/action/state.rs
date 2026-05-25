use super::*;

#[derive(Debug, Clone)]
pub enum EditorStateAction {
    Exit,
    Editor(EditorAction),
    Cancel,
    StopTextEdit,
    UpdateTextEdit(String),
    CursorMove(vec2<f32>),
    WheelScroll(f32),
    StartPlaytest,
    EndDrag,
    StartDrag(DragTarget),
    ConfirmPopupAction,
    ContextMenu(vec2<f32>, Vec<(Name, EditorStateAction)>),
    CloseContextMenu,

    SelectMusicFile(std::path::PathBuf),
    SetGroupName(String),
    AddMusicAuthor(MusicianInfo),
    UpdateMusicAuthor(usize, MusicianInfo),
    RemoveMusicAuthor(usize),

    SetLevelName(usize, Name),
    AddLevelAuthor(usize, MapperInfo),
    UpdateLevelAuthor(usize, usize, MapperInfo),
    RemoveLevelAuthor(usize, usize),
}

impl From<EditorAction> for EditorStateAction {
    fn from(value: EditorAction) -> Self {
        Self::Editor(value)
    }
}

impl From<LevelAction> for EditorStateAction {
    fn from(value: LevelAction) -> Self {
        Self::Editor(EditorAction::Level(value))
    }
}
