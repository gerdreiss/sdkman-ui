use candidates::*;
use eframe::egui::CentralPanel;
use eframe::egui::CtxRef;
use eframe::egui::Hyperlink;
use eframe::egui::Label;
use eframe::egui::ScrollArea;
use eframe::egui::TextStyle;
use eframe::egui::TopBottomPanel;
use eframe::egui::Vec2;
use eframe::epi::App;
use eframe::run_native;
use eframe::NativeOptions;

mod candidates;

impl App for Candidates {
    fn setup(
        &mut self,
        ctx: &eframe::egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.configure_fonts(ctx);
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, _frame: &mut eframe::epi::Frame<'_>) {
        self.render_top_panel(ctx);
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::auto_sized().show(ui, |ui| {
                self.render_candidates(ui);
            });
            render_footer(ctx);
        });
    }

    fn name(&self) -> &str {
        "Candidates"
    }
}

fn render_footer(ctx: &CtxRef) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.add(Label::new("API source: https://api.sdkman.io/2").monospace());
            ui.add(
                Hyperlink::new("https://github.com/emilk/egui")
                    .text("Made with egui")
                    .text_style(TextStyle::Monospace),
            );
            ui.add(
                Hyperlink::new("https://github.com/gerdreiss/sdkmanr")
                    .text("gerdreiss/sdkmanr")
                    .text_style(TextStyle::Monospace),
            );
            ui.add_space(10.);
        })
    });
}

fn main() {
    let candidates = api::fetch_candidates().unwrap();
    let app = Candidates::new(&candidates);
    let mut win_option = NativeOptions::default();
    win_option.initial_window_size = Some(Vec2::new(1024., 960.));
    run_native(Box::new(app), win_option);
}
