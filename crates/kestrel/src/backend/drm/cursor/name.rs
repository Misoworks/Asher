use smithay::input::pointer::{CursorIcon, CursorImageStatus};

pub(super) fn cursor_name(image: &CursorImageStatus) -> &'static str {
    let CursorImageStatus::Named(icon) = image else {
        return "default";
    };
    match icon {
        CursorIcon::Pointer => "pointer",
        CursorIcon::Text => "text",
        CursorIcon::Grab => "grab",
        CursorIcon::Grabbing => "grabbing",
        CursorIcon::EResize | CursorIcon::WResize | CursorIcon::EwResize => "ew-resize",
        CursorIcon::NResize | CursorIcon::SResize | CursorIcon::NsResize => "ns-resize",
        CursorIcon::NeResize | CursorIcon::SwResize | CursorIcon::NeswResize => "nesw-resize",
        CursorIcon::NwResize | CursorIcon::SeResize | CursorIcon::NwseResize => "nwse-resize",
        CursorIcon::Crosshair => "crosshair",
        CursorIcon::Wait | CursorIcon::Progress => "wait",
        CursorIcon::Help => "help",
        CursorIcon::ZoomIn => "zoom-in",
        CursorIcon::ZoomOut => "zoom-out",
        CursorIcon::NotAllowed | CursorIcon::NoDrop => "no-drop",
        CursorIcon::Copy => "copy",
        CursorIcon::Alias => "alias",
        CursorIcon::AllScroll => "all-scroll",
        CursorIcon::Cell => "cell",
        CursorIcon::ContextMenu => "context-menu",
        CursorIcon::VerticalText => "vertical-text",
        _ => "default",
    }
}
