use bevy_egui::egui::{self, emath::Numeric};

pub fn edit_slice(v: &mut [impl Numeric]) -> impl egui::Widget + '_ {
    edit_slice_impl(v, |d| d.speed(0.1))
}

fn edit_slice_impl<'a>(
    v: &'a mut [impl Numeric],
    c: impl Fn(egui::DragValue) -> egui::DragValue + 'a,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let mut response = None;
        ui.horizontal(|ui| {
            for v in v {
                let r = ui.add(c(egui::DragValue::new(v)));
                match &mut response {
                    Some(response) => *response |= r,
                    None => response = Some(r),
                }
            }
        });
        response.expect("Slice must have at least 1 value to edit")
    }
}
