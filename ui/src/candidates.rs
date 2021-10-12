use api::*;
use eframe::egui::{
    self, Button, Color32, CtxRef, FontDefinitions, FontFamily, Hyperlink, Label, Layout, Sense,
    Separator, TopBottomPanel,
};
use std::borrow::Cow;

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
    fn new(model: &CandidateModel) -> Candidate {
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
    candidates: Vec<Candidate>,
}

impl Candidates {
    pub fn new(models: &Vec<CandidateModel>) -> Candidates {
        Candidates {
            candidates: models.iter().map(|model| Candidate::new(model)).collect(),
        }
    }

    pub fn configure_fonts(&self, ctx: &CtxRef) {
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

    pub(crate) fn render_top_panel(&self, ctx: &CtxRef) {
        // define a TopBottomPanel widget
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.);
            egui::menu::bar(ui, |ui| {
                // logo
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📓").text_style(egui::TextStyle::Heading));
                });
                // Candidates
                ui.vertical_centered(|ui| {
                    ui.heading("Candidates");
                });
                // controls
                ui.with_layout(Layout::right_to_left(), |ui| {
                    let _close_btn = ui.add(Button::new("❌").text_style(egui::TextStyle::Body));
                    let _refresh_btn = ui.add(Button::new("🔄").text_style(egui::TextStyle::Body));
                    let _theme_btn = ui.add(Button::new("🌙").text_style(egui::TextStyle::Body));
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
                        .text_style(egui::TextStyle::Body)
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
}
