use candidates::Candidates;
use eframe::egui::CentralPanel;
use eframe::egui::CtxRef;
use eframe::egui::FontDefinitions;
use eframe::egui::FontFamily;
use eframe::egui::ScrollArea;
use eframe::egui::Vec2;
use eframe::epi::App;
use eframe::run_native;
use eframe::NativeOptions;
use std::borrow::Cow;

mod candidates;

impl App for Candidates {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        configure_fonts(ctx);
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, _frame: &mut eframe::epi::Frame<'_>) {
        self.render_top_panel(ctx);
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::auto_sized().show(ui, |ui| {
                self.render_candidates(ui);
            });
            self.render_footer(ctx);
        });
    }

    fn name(&self) -> &str {
        self.app_name()
    }
}

pub fn configure_fonts(ctx: &CtxRef) {
    let mut font_def = FontDefinitions::default();
    font_def.font_data.insert(
        "MesloLGS".to_string(),
        Cow::Borrowed(include_bytes!("../assets/MesloLGS_NF_Regular.ttf")),
    );
    font_def.family_and_size.insert(
        eframe::egui::TextStyle::Heading,
        (FontFamily::Proportional, 35.),
    );
    font_def.family_and_size.insert(
        eframe::egui::TextStyle::Body,
        (FontFamily::Proportional, 20.),
    );
    font_def
        .fonts_for_family
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "MesloLGS".to_string());
    ctx.set_fonts(font_def);
}

fn main() {
    let candidates = api::fetch_candidates().unwrap();
    let app = Candidates::new(&candidates);
    let mut win_option = NativeOptions::default();
    win_option.initial_window_size = Some(Vec2::new(1024., 960.));
    run_native(Box::new(app), win_option);
}
