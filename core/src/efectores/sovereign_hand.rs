use anyhow::Result;
use enigo::{Enigo, KeyboardControllable, MouseButton as EnigoButton, MouseControllable};

#[derive(Clone)]
pub struct NexusHand;

impl Default for NexusHand {
    fn default() -> Self {
        Self::new()
    }
}

impl NexusHand {
    pub fn new() -> Self {
        Self
    }

    pub fn wake() -> Result<Self> {
        Ok(Self::new())
    }

    pub fn move_mouse(&self, x: i32, y: i32, absolute: bool) -> Result<()> {
        let mut enigo = Enigo::new();
        if absolute {
            enigo.mouse_move_to(x, y);
        } else {
            enigo.mouse_move_relative(x, y);
        }
        Ok(())
    }

    pub fn move_relative(&self, dx: i32, dy: i32) -> Result<()> {
        let mut enigo = Enigo::new();
        enigo.mouse_move_relative(dx, dy);
        Ok(())
    }

    pub fn click(&self, button: MouseButton) -> Result<()> {
        let mut enigo = Enigo::new();
        let btn = match button {
            MouseButton::Left => EnigoButton::Left,
            MouseButton::Right => EnigoButton::Right,
            MouseButton::Middle => EnigoButton::Middle,
        };
        enigo.mouse_click(btn);
        Ok(())
    }

    pub fn type_text(&self, text: &str) -> Result<()> {
        let mut enigo = Enigo::new();
        enigo.key_sequence(text);
        Ok(())
    }

    pub fn press_key(&self, key: enigo::Key) -> Result<()> {
        let mut enigo = Enigo::new();
        enigo.key_click(key);
        Ok(())
    }
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}
