use csv::{self, StringRecord};
use egui::{pos2, Align2, FontId, NumExt as _, Rect, RichText, ScrollArea, Sense};
use std::collections::HashMap;

static NEUROPAL_ORG: &[u8] = include_bytes!("neuropal.csv");
static NEUROPAL_HEADER: [&str; 7] = ["name", "x", "y", "z", "r", "g", "b"];
#[derive(serde::Deserialize)]
struct Neuron {
    name: String,
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    g: f32,
    b: f32,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MyApp {
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    data: HashMap<String, Neuron>,
}

impl Default for MyApp {
    fn default() -> Self {
        let header = StringRecord::from(NEUROPAL_HEADER.to_vec());

        let data = csv::ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(NEUROPAL_ORG)
            .records()
            .filter_map(|x| x.ok())
            .filter_map(|r| r.deserialize::<Neuron>(Some(&header)).ok())
            .map(|x| (x.name.to_owned(), x))
            .collect();

        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            data,
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for MyApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(20.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("NeuroPAL Palette");

            ui.horizontal(|ui| {
                ui.label("Search: ");
                ui.text_edit_singleline(&mut self.label);
            });
            ui.label(RichText::new("Name  (x, y, z)").font(FontId::monospace(16.0)));
            let mut data: Vec<_> = self
                .data
                .values()
                .filter(|x| {
                    [" ", ";", ","].iter().any(|delimiter| {
                        self.label
                            .split(delimiter)
                            .filter(|x| !x.is_empty())
                            .any(|pat| pat == "*" || x.name.starts_with(pat))
                    })
                })
                .collect();
            data.sort_unstable_by_key(|x| &x.name);
            huge_content_painter(ui, data);

            ui.add(egui::github_link_file!(
                "https://github.com/lycantrope/NeuroPALette/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

fn huge_content_painter(ui: &mut egui::Ui, data: Vec<&Neuron>) {
    ui.add_space(4.0);
    let font_id = FontId::monospace(16.0);
    let row_height = ui.fonts(|f| f.row_height(&font_id)) + ui.spacing().item_spacing.y;

    let num_rows = data.len();
    ScrollArea::vertical()
        .auto_shrink(false)
        .show_viewport(ui, |ui, viewport| {
            ui.set_height(row_height * num_rows as f32);

            let first_item = (viewport.min.y / row_height).floor().at_least(0.0) as usize;
            let last_item = (viewport.max.y / row_height).ceil() as usize + 1;
            let last_item = last_item.at_most(num_rows);

            let mut used_rect = Rect::NOTHING;

            for i in first_item..last_item {
                let x = ui.min_rect().left();
                let y = ui.min_rect().top() + i as f32 * row_height;
                if let Some(neuron) = data.get(i) {
                    let text = neuron.name.as_str();
                    let (r, g, b) = (neuron.r * 255., neuron.g * 255., neuron.b * 255.);

                    let lut = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    let (mut r, mut g, mut b) = (r as u8, g as u8, b as u8);

                    if r == 0 && g == 0 && b == 0 {
                        r = 255;
                        g = 255;
                        b = 255;
                    }
                    let text_color = if lut == 0.0 || lut > 112.5 {
                        egui::Color32::BLACK
                    } else {
                        egui::Color32::WHITE
                    };
                    ui.painter().rect(
                        Rect::from_min_max(pos2(x, y), pos2(x + 240., y + row_height)),
                        0.0f32,
                        egui::Color32::from_rgb(r, g, b),
                        (0.0, egui::Color32::from_rgb(r, g, b)),
                    );
                    let text_rect = ui.painter().text(
                        pos2(x, y),
                        Align2::LEFT_TOP,
                        format!(
                            "{:<5} ({:.1},{:.1},{:.1})",
                            text, neuron.x, neuron.y, neuron.z
                        ),
                        font_id.clone(),
                        text_color,
                    );
                    used_rect = used_rect.union(text_rect);
                }
            }

            ui.allocate_rect(used_rect, Sense::hover()); // make sure it is visible!
        });
}
