//! A GUI interface for fmn-passgen
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// provides:
// static WORDLIST: &[&str] = &[...]
include!(concat!(env!("OUT_DIR"), "/wordlist.rs"));

use std::sync::Arc;

use eframe::egui;
use egui::Color32;
use egui::RichText;
use rand::Rng;
use rand::SeedableRng;
use rand::TryRngCore;
use rand::rngs::OsRng;
use rand::rngs::SmallRng;

use fmn_passgen::config::Config;
use fmn_passgen::config::ConfigBuilder;
use fmn_passgen::password_maker::PasswordMaker;
use fmn_passgen::types::PaddingType;
use fmn_passgen::types::StrEnum;
use fmn_passgen::types::WordTransformationType;

const INITIAL_SEED: u64 = 13414357264162109690;
const MIN_WIDTH: f32 = 400.0;
const MIN_HEIGHT: f32 = 740.0;
const BUTTON_FULL_WIDTH: f32 = 384.0;
const DEFAULT_SPACING: f32 = 17.0;
const COMPACT_SPACING: f32 = 3.0;

struct App {
    config_prev: Config,
    config_curr: Config,
    password_maker: PasswordMaker<OsRng>,
    preview_maker: PasswordMaker<SmallRng>,
    preview: String,
    generated_passwords: Vec<String>,
    padding_characters: String,
    separator_characters: String,
    seed_string: String,
    curr_seed: u64,
    prev_seed: u64,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(DEFAULT_SPACING, DEFAULT_SPACING);
            style.interaction.selectable_labels = false;
        });
        let config_prev = ConfigBuilder::new().build().unwrap();
        let config_curr = config_prev.clone();
        // TODO take references to a config?
        let password_maker = PasswordMaker::<OsRng>::new(config_curr.clone());
        let seed_string = INITIAL_SEED.to_string();
        let curr_seed = INITIAL_SEED;
        let prev_seed = INITIAL_SEED;
        let mut preview_maker: PasswordMaker<SmallRng> = PasswordMaker {
            rng: SmallRng::seed_from_u64(curr_seed).unwrap_err(),
            config: config_curr.clone(),
            wordlist: WORDLIST.iter().map(|s| String::from(*s)).collect(),
        };
        let preview: String = preview_maker.make_password();
        let generated_passwords: Vec<String> = Vec::new();
        let padding_characters: String = config_curr.padding_characters.iter().collect();
        let separator_characters: String = config_curr.separator_characters.iter().collect();
        Self {
            config_prev,
            config_curr,
            password_maker,
            seed_string,
            curr_seed,
            prev_seed,
            preview_maker,
            preview,
            generated_passwords,
            padding_characters,
            separator_characters,
        }
    }
    fn show_inner(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::CollapsingHeader::new("preview")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let response = ui.add_sized(
                        [160.0, DEFAULT_SPACING],
                        egui::TextEdit::singleline(&mut self.seed_string),
                    );
                    if response.changed() {
                        if let Ok(seed) = self.seed_string.parse::<u64>() {
                            self.curr_seed = seed;
                        } else {
                            self.seed_string = self.curr_seed.to_string();
                        }
                    }
                    if ui.button("random seed").clicked() {
                        self.curr_seed = self.password_maker.rng.random();
                        self.seed_string = self.curr_seed.to_string();
                    }
                });
                if self.config_curr != self.config_prev || self.curr_seed != self.prev_seed {
                    self.preview_maker.rng = SmallRng::seed_from_u64(self.curr_seed).unwrap_err();
                    self.preview_maker.config = self.config_curr.clone();
                    self.preview = self.preview_maker.make_password();
                    self.config_prev = self.config_curr.clone();
                    self.prev_seed = self.curr_seed;
                }
                ui.label(RichText::new(&self.preview).color(Color32::ORANGE));
            });
        egui::CollapsingHeader::new("words")
            .default_open(true)
            .show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.config_curr.word_count, 0..=32)
                        .text("count")
                        .logarithmic(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.config_curr.word_min_length, 3..=9)
                        .text("min length"),
                );
                ui.add(
                    egui::Slider::new(
                        &mut self.config_curr.word_max_length,
                        self.config_curr.word_min_length..=9,
                    )
                    .text("max length"),
                );
                egui::ComboBox::from_label("transform")
                    .selected_text(self.config_curr.word_transformation.to_static_str())
                    .show_ui(ui, |ui| {
                        ui.style_mut().spacing.item_spacing =
                            egui::vec2(COMPACT_SPACING, COMPACT_SPACING);
                        for (description, item) in WordTransformationType::NAME_MEMBER_ARR {
                            ui.selectable_value(
                                &mut self.config_curr.word_transformation,
                                *item,
                                *description,
                            );
                        }
                    });
            });
        egui::CollapsingHeader::new("digits")
            .default_open(true)
            .show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.config_curr.digits_before, 0..=255)
                        .text("before")
                        .logarithmic(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.config_curr.digits_after, 0..=255)
                        .text("after")
                        .logarithmic(true),
                );
            });
        egui::CollapsingHeader::new("padding")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let name_label = ui.label("possible choices");
                    ui.text_edit_singleline(&mut self.padding_characters)
                        .labelled_by(name_label.id);
                });
                self.config_curr.padding_characters = self.padding_characters.chars().collect();
                egui::ComboBox::from_label("type")
                    .selected_text(self.config_curr.padding_type.to_static_str())
                    .show_ui(ui, |ui| {
                        ui.style_mut().spacing.item_spacing =
                            egui::vec2(COMPACT_SPACING, COMPACT_SPACING);
                        for (description, item) in PaddingType::NAME_MEMBER_ARR {
                            // displaying aliases is unnecessary
                            if matches!(item, PaddingType::Fixed | PaddingType::Adaptive) {
                                continue;
                            }
                            ui.selectable_value(
                                &mut self.config_curr.padding_type,
                                *item,
                                *description,
                            );
                        }
                    });
                ui.add(
                    egui::Slider::new(&mut self.config_curr.padding_length, 0..=255)
                        .text("length")
                        .logarithmic(true),
                );
            });
        egui::CollapsingHeader::new("separator")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let name_label = ui.label("possible choices");
                    ui.text_edit_singleline(&mut self.separator_characters)
                        .labelled_by(name_label.id);
                });
                self.config_curr.separator_characters = self.separator_characters.chars().collect();
            });
        ui.add(
            egui::Slider::new(&mut self.config_curr.count, 1..=255)
                .text("how many to generate")
                .logarithmic(true),
        );

        if ui
            .add_sized(
                [BUTTON_FULL_WIDTH, DEFAULT_SPACING],
                egui::Button::new("generate").fill(Color32::DARK_GREEN),
            )
            .clicked()
        {
            self.password_maker.config = self.config_curr.clone();
            self.generated_passwords = self.password_maker.make_passwords();
        }

        for item in self.generated_passwords.iter() {
            ui.horizontal(|ui| {
                if ui.button("📋").clicked() {
                    ctx.copy_text(item.clone());
                }
                ui.add(egui::Label::new(item).selectable(true).wrap());
            });
        }
        ui.allocate_space(ui.available_size());
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.show_inner(ui, ctx));
        });
    }
}

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../icon.png")).unwrap();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size([MIN_WIDTH, MIN_HEIGHT])
            .with_inner_size([MIN_WIDTH, MIN_HEIGHT])
            .with_icon(Arc::new(icon)),
        ..Default::default()
    };
    eframe::run_native(
        "fmn-passgen",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(cc)))
        }),
    )
}
