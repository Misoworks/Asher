use super::{ClientGrabSerial, KestrelState};
use smithay::{input::keyboard::LedState, utils::Serial, wayland::shell::xdg::ToplevelSurface};

impl KestrelState {
    pub fn next_serial(&mut self) -> Serial {
        let serial = self.serial;
        self.serial = self.serial.wrapping_add(1).max(1);
        Serial::from(serial)
    }

    pub fn allow_client_grab(&mut self, surface: ToplevelSurface, _serial: Serial) {
        self.pending_client_grab = Some(ClientGrabSerial { surface });
    }

    pub fn clear_client_grab(&mut self) {
        self.pending_client_grab = None;
    }

    pub fn client_grab_allowed(&self, surface: &ToplevelSurface, _serial: Serial) -> bool {
        self.pending_client_grab
            .as_ref()
            .is_some_and(|grab| &grab.surface == surface)
    }

    pub(crate) fn set_pending_keyboard_led_state(&mut self, led_state: LedState) {
        self.pending_keyboard_led_state = Some(led_state);
    }

    #[cfg(feature = "session-backend")]
    pub(crate) fn take_pending_keyboard_led_state(&mut self) -> Option<LedState> {
        self.pending_keyboard_led_state.take()
    }
}
