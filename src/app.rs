use csv::{self, StringRecord};
use egui::{
    pos2, Align2, Button, Color32, FontId, NumExt as _, Rect, RichText, ScrollArea, Sense, Theme,
};
use egui_plot::{HLine, PlotPoints, Points, Text, VLine};

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

impl Neuron {
    pub fn rgb(&self) -> [u8; 3] {
        [
            (self.r * 255.).clamp(0., 255.) as u8,
            (self.g * 255.).clamp(0., 255.) as u8,
            (self.b * 255.).clamp(0., 255.) as u8,
        ]
    }
    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}

#[inline]
fn l2_dist(x1: f64, x2: f64, y1: f64, y2: f64) -> f64 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}
#[derive(serde::Deserialize, serde::Serialize)]
enum WormSide {
    Left,
    Right,
    Both,
}

impl WormSide {
    fn next(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Both,
            Self::Both => Self::Left,
        }
    }
    fn color(&self) -> Color32 {
        match self {
            Self::Left => Color32::from_rgba_unmultiplied(131, 240, 22, 120),
            Self::Right => Color32::from_rgba_unmultiplied(240, 22, 131, 120),
            Self::Both => Color32::from_rgba_unmultiplied(22, 131, 240, 120),
        }
    }
}

impl std::fmt::Display for WormSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
            Self::Both => write!(f, "Both"),
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MyApp {
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    data: HashMap<String, Neuron>,

    show_side_panel: bool,
    view_side: WormSide,
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
            label: "*".to_owned(),
            data,
            show_side_panel: true,
            view_side: WormSide::Both,
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
                    ui.separator();
                }
                egui::widgets::global_theme_preference_switch(ui);
                ui.separator();
                let mut btn = Button::new(RichText::new("Filter Panel").monospace());
                if self.show_side_panel {
                    btn = btn.fill(Color32::from_rgba_unmultiplied(22, 131, 240, 120));
                };
                if ui.add(btn).clicked() {
                    self.show_side_panel = !self.show_side_panel;
                };
                egui::warn_if_debug_build(ui);
            });
        });

        let mut data: Vec<_> = self
            .data
            .values()
            .filter(|x| {
                self.label
                    .split(&[' ', ';', ',', '\t'])
                    .filter(|x| !x.is_empty())
                    .any(|pat| pat == "*" || x.name.starts_with(pat))
            })
            .filter(|x| match self.view_side {
                WormSide::Left => x.z >= 0.,
                WormSide::Right => x.z < 0.,
                WormSide::Both => true,
            })
            .collect();
        data.sort_unstable_by_key(|x| &x.name);

        if self.show_side_panel {
            egui::SidePanel::left("SideTool").show(ctx, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("NeuroPAL Lens").strong());
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("(");
                    ui.hyperlink_to(
                        "\u{E624} Source code.",
                        "https://github.com/lycantrope/neuropal_lens",
                    );
                    ui.label(")");
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Body Side:").heading());
                    let btn = egui::Button::new(
                        RichText::new(self.view_side.to_string()).heading().strong(),
                    )
                    .min_size([180., 20.].into());
                    let btn = btn.fill(self.view_side.color());
                    if ui.add(btn).clicked() {
                        self.view_side = self.view_side.next();
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Search: ");
                    ui.text_edit_singleline(&mut self.label);
                });
                ui.label(
                    RichText::new(" Name  (    x,     y,     z)").font(FontId::monospace(16.0)),
                );

                huge_content_painter(ui, &data);
            });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            worm_canvas(ctx, ui, &data);
        });
    }
}

fn huge_content_painter(ui: &mut egui::Ui, data: &[&Neuron]) {
    ui.add_space(4.0);
    let font_id = FontId::monospace(16.0);
    let row_height = ui.fonts(|f| f.row_height(&font_id)) + ui.spacing().item_spacing.y;
    let row_width = ui.fonts(|f| f.glyph_width(&font_id, 'X')) * 28. + ui.spacing().item_spacing.x;
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
                let x = ui.min_rect().left() + ui.spacing().item_spacing.x;
                let y = ui.min_rect().top() + i as f32 * row_height;
                if let Some(neuron) = data.get(i) {
                    let text = neuron.name.as_str();
                    let (r, g, b) = (neuron.r * 255., neuron.g * 255., neuron.b * 255.);

                    let lut = neuron.luminance();
                    let (mut r, mut g, mut b) = (r as u8, g as u8, b as u8);

                    if r == 0 && g == 0 && b == 0 {
                        r = 255;
                        g = 255;
                        b = 255;
                    }
                    let text_color = if lut == 0.0 || lut > 0.5 {
                        egui::Color32::BLACK
                    } else {
                        egui::Color32::WHITE
                    };
                    ui.painter().rect(
                        Rect::from_min_max(pos2(x, y), pos2(x + row_width, y + row_height)),
                        0.0f32,
                        egui::Color32::from_rgb(r, g, b),
                        (0.0, egui::Color32::from_rgb(r, g, b)),
                    );
                    let text_rect = ui.painter().text(
                        pos2(x, y),
                        Align2::LEFT_TOP,
                        format!(
                            "{:<5} ({:>5.1}, {:>5.1}, {:>5.1})",
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

fn worm_canvas(ctx: &egui::Context, ui: &mut egui::Ui, data: &[&Neuron]) {
    let is_dark = ui.ctx().theme() == Theme::Dark;
    let response = egui_plot::Plot::new("xy")
        .height(500.)
        .data_aspect(1.0)
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .allow_double_click_reset(true)
        .allow_boxed_zoom(true)
        .include_x(0.0)
        .include_y(0.0)
        // .legend(Legend::default())
        .x_axis_label(RichText::new("Anterior - Posterior").strong())
        .y_axis_label(RichText::new("Ventral - Dorsal").strong())
        .show(ui, |plot_ui| {
            let boundary = plot_ui.plot_bounds();
            let scale = boundary.max()[0] - boundary.min()[0];
            let radius = (scale * -0.01 + 6.).clamp(1.0, 6.);

            for neuron in data {
                let pts = vec![[neuron.x as f64, neuron.y as f64]];
                let points = PlotPoints::new(pts);
                let [r, g, b] = neuron.rgb();

                let mut color = if r == 0 && g == 0 && b == 0 && is_dark {
                    egui::Color32::WHITE
                } else {
                    let [r, g, b] = neuron.rgb();
                    egui::Color32::from_rgb(r, g, b)
                };

                if neuron.z < 0.0 {
                    color = color.gamma_multiply(0.8);
                }

                plot_ui.points(
                    Points::new(points)
                        .name(&neuron.name)
                        .allow_hover(true)
                        .color(color)
                        .highlight(true)
                        .radius(radius as f32),
                );
            }
        });

    let pos = response
        .response
        .hover_pos()
        .map(|pos| response.transform.value_from_position(pos));

    let thickness = 1.5;
    let bound = response.transform.bounds();
    let x_bound = (bound.min()[0], bound.max()[0]);
    let yz_window = egui::Window::new("Anterior View (z-y)")
        .id(egui::Id::new("yz")) // required since we change the title
        .resizable(true)
        .constrain(true)
        .collapsible(true)
        .title_bar(true)
        .scroll(true)
        .enabled(true);

    yz_window.show(ctx, |ui| {
        egui_plot::Plot::new("yz")
            .data_aspect(1.0)
            .allow_zoom(true)
            .allow_drag(true)
            .allow_scroll(true)
            .allow_double_click_reset(true)
            .allow_boxed_zoom(true)
            .include_x(-15.0)
            .include_x(15.0)
            .include_y(20.0)
            .include_y(-25.0)
            .x_axis_label(RichText::new("Right - Left").strong())
            .y_axis_label(RichText::new("Ventral - Dorsal").strong())
            // .legend(Legend::default())
            .show(ui, |plot_ui| {
                let boundary = plot_ui.plot_bounds();
                let scale = boundary.max()[0] - boundary.min()[0];
                let radius = (scale * -0.01 + 6.).clamp(1.0, 6.);
                let mut low = f64::MIN;
                let mut high = f64::MAX;
                if let Some(pos) = pos {
                    plot_ui.hline(HLine::new(pos.y).color(Color32::LIGHT_RED));
                    low = pos.x - thickness;
                    high = pos.x + thickness;
                }

                for neuron in data {
                    let x_pos = neuron.x as f64;
                    if x_pos < low || x_pos > high {
                        continue;
                    }
                    let pts = [neuron.z as f64, neuron.y as f64];

                    let points = PlotPoints::new(vec![pts]);
                    let [r, g, b] = neuron.rgb();

                    let mut color = if r == 0 && g == 0 && b == 0 && is_dark {
                        egui::Color32::WHITE
                    } else {
                        let [r, g, b] = neuron.rgb();
                        egui::Color32::from_rgb(r, g, b)
                    };

                    if neuron.z < 0.0 {
                        color = color.gamma_multiply(0.8);
                    }

                    plot_ui.points(
                        Points::new(points)
                            .name(&neuron.name)
                            .allow_hover(true)
                            .color(color)
                            .highlight(true)
                            .radius(radius as f32),
                    );

                    if pos.is_some_and(|pos| {
                        l2_dist(neuron.x as f64, pos.x, neuron.y as f64, pos.y) < 0.35
                    }) {
                        plot_ui.vline(VLine::new(neuron.z).color(Color32::LIGHT_RED));
                        let points = PlotPoints::new(vec![pts]);
                        plot_ui.points(
                            Points::new(points)
                                .color(egui::Color32::LIGHT_RED)
                                .filled(false)
                                .radius(radius as f32 + 2.0),
                        );
                        let text_pos = [
                            neuron.z as f64 + radius / 1.5,
                            neuron.y as f64 + radius / 1.5,
                        ]
                        .into();

                        plot_ui.text(Text::new(text_pos, &neuron.name).highlight(true));
                    }
                }
            });
    });
    let xz_window = egui::Window::new("Dorsal View (x-z)")
        .id(egui::Id::new("xz")) // required since we change the title
        .resizable(true)
        .constrain(true)
        .collapsible(true)
        .title_bar(true)
        .scroll(true)
        .enabled(true);

    xz_window.show(ctx, |ui| {
        egui_plot::Plot::new("xz")
            .data_aspect(1.0)
            .allow_zoom(true)
            .allow_drag(true)
            .allow_scroll(true)
            .allow_double_click_reset(true)
            .allow_boxed_zoom(true)
            .include_x(x_bound.0)
            .include_x(x_bound.1)
            .include_y(15.0)
            .include_y(-15.0)
            .x_axis_label(RichText::new("Anterior - Posterior").strong())
            .y_axis_label(RichText::new("Left - Right").strong())
            .show(ui, |plot_ui| {
                let boundary = plot_ui.plot_bounds();
                let scale = boundary.max()[0] - boundary.min()[0];
                let radius = (scale * -0.01 + 6.).clamp(1.0, 6.);

                let mut y_min = f64::MIN;
                let mut y_max = f64::MAX;
                let (x_min, x_max) = x_bound;

                if let Some(pos) = pos {
                    plot_ui.vline(VLine::new(pos.x).color(Color32::LIGHT_RED));
                    y_min = pos.y - thickness;
                    y_max = pos.y + thickness;
                }
                for neuron in data {
                    let x_pos = neuron.x as f64;
                    let y_pos = neuron.y as f64;
                    if y_pos < y_min || y_pos > y_max || x_pos < x_min || x_pos > x_max {
                        continue;
                    }

                    let pts = [neuron.x as f64, -neuron.z as f64];

                    let points = PlotPoints::new(vec![pts]);
                    let [r, g, b] = neuron.rgb();

                    let mut color = if r == 0 && g == 0 && b == 0 && is_dark {
                        egui::Color32::WHITE
                    } else {
                        let [r, g, b] = neuron.rgb();
                        egui::Color32::from_rgb(r, g, b)
                    };

                    if neuron.z < 0.0 {
                        color = color.gamma_multiply(0.8);
                    }

                    plot_ui.points(
                        Points::new(points)
                            .name(&neuron.name)
                            .allow_hover(true)
                            .color(color)
                            .highlight(true)
                            .radius(radius as f32),
                    );

                    if pos.is_some_and(|pos| {
                        l2_dist(neuron.x as f64, pos.x, neuron.y as f64, pos.y) < 0.35
                    }) {
                        plot_ui.hline(HLine::new(-neuron.z).color(Color32::LIGHT_RED));
                        let points = PlotPoints::new(vec![pts]);
                        plot_ui.points(
                            Points::new(points)
                                .color(egui::Color32::LIGHT_RED)
                                .filled(false)
                                .radius(radius as f32 + 2.0),
                        );
                        let text_pos = [
                            neuron.x as f64 + radius / 1.5,
                            -neuron.z as f64 + radius / 1.5,
                        ]
                        .into();

                        plot_ui.text(Text::new(text_pos, &neuron.name).highlight(true));
                    }
                }
            });
    });
}
