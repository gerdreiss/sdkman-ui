use eframe::egui::menu;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::CtxRef;
use eframe::egui::Hyperlink;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::Sense;
use eframe::egui::Separator;
use eframe::egui::TextStyle;
use eframe::egui::TopBottomPanel;

const PADDING: f32 = 8.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);

pub struct Candidate {
    name: String,
    default_version: String,
    url: String,
    description: String,
    installation: String,
}

impl Candidate {
    fn from_model(model: &api::CandidateModel) -> Candidate {
        Candidate {
            name: model.name.clone(),
            default_version: model.default_version.clone(),
            url: model.homepage.clone(),
            description: model.description.clone(),
            installation: format!("$ sdk install {}", model.binary),
        }
    }
}

pub struct Candidates {
    app_name: &'static str,
    app_heading: &'static str,
    candidates: Vec<Candidate>,
}

impl Candidates {
    pub fn new(models: &Vec<api::CandidateModel>) -> Candidates {
        Candidates {
            app_name: "sdkman-ui",
            app_heading: "candidates",
            candidates: models
                .iter()
                .map(|model| Candidate::from_model(model))
                .collect(),
        }
    }

    pub fn app_name(&self) -> &str {
        self.app_name
    }

    pub fn app_heading(&self) -> &str {
        self.app_heading
    }

    pub(crate) fn render_top_panel(&self, ctx: &CtxRef) {
        // define a TopBottomPanel widget
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            menu::bar(ui, |ui| {
                // logo
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📓").text_style(TextStyle::Heading));
                });
                // Candidates
                ui.vertical_centered(|ui| {
                    ui.heading(self.app_heading());
                });
                // controls
                ui.with_layout(Layout::right_to_left(), |ui| {
                    let _close_btn = ui.add(Button::new("❌").text_style(TextStyle::Body));
                    let _refresh_btn = ui.add(Button::new("🔄").text_style(TextStyle::Body));
                    let _theme_btn = ui.add(Button::new("🌙").text_style(TextStyle::Body));
                });
            });
            ui.add_space(10.);
        });
    }

    pub fn render_candidates(&self, ui: &mut eframe::egui::Ui) {
        for a in &self.candidates {
            ui.add_space(PADDING);

            ui.horizontal(|ui| {
                // render name
                ui.with_layout(Layout::left_to_right(), |ui| {
                    let name = Label::new(format!("{} {} ⤴", a.name, a.default_version))
                        .text_style(TextStyle::Body)
                        .text_color(WHITE)
                        .sense(Sense::click());
                    ui.add(name);
                });
                // render URL
                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.style_mut().visuals.hyperlink_color = CYAN;
                    ui.add(Hyperlink::new(&a.url).text(&a.url));
                });
            });

            ui.add_space(PADDING);

            // render description
            let description = Label::new(&a.description)
                .wrap(true)
                .text_style(eframe::egui::TextStyle::Body);
            ui.add(description);

            ui.add_space(PADDING);

            // render installation
            ui.with_layout(Layout::right_to_left(), |ui| {
                let installation =
                    Label::new(&a.installation).text_style(eframe::egui::TextStyle::Body);
                ui.add(installation);
            });
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }

    pub fn render_footer(&self, ctx: &CtxRef) {
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
                    Hyperlink::new("https://github.com/gerdreiss/sdkman-ui")
                        .text("gerdreiss/sdkman-ui")
                        .text_style(TextStyle::Monospace),
                );
                ui.add_space(10.);
            })
        });
    }
}
