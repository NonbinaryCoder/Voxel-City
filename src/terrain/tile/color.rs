use std::num::NonZeroU8;

use bevy_egui::egui::{self, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexedColor(NonZeroU8);

impl Default for IndexedColor {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl IndexedColor {
    // Safety: the value is not zero
    pub const DEFAULT: Self = Self(unsafe { NonZeroU8::new_unchecked(1) });

    pub const MAX_INDEX: u8 = 61;

    pub fn uv(self) -> [f32; 2] {
        let x = (self.index() % 8) as f32 + 0.5;
        let y = (self.index() / 8) as f32 + 0.5;
        [x / 8.0, y / 8.0]
    }

    pub fn index(self) -> u8 {
        self.0.get() - 1
    }

    pub fn from_index(i: u8) -> Option<Self> {
        (i <= Self::MAX_INDEX).then(|| Self(NonZeroU8::new(i + 1).unwrap()))
    }
}

impl Widget for &mut IndexedColor {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut index = self.index();
        let response =
            ui.add(egui::DragValue::new(&mut index).clamp_range(0..=IndexedColor::MAX_INDEX));
        *self = IndexedColor::from_index(index).unwrap();
        response
    }
}
