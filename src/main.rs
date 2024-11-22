use eframe::egui;
use egui::{Context, ScrollArea, Ui};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
struct CustomStructure {
    name: String,
    fields: Vec<String>,
}

#[derive(Default)]
struct ERPApp {
    custom_structures: Vec<CustomStructure>,
    current_structure: CustomStructure,
    show_setting_panel: bool,
}

impl ERPApp {
    fn load_custom_structures(&mut self) {
        if let Ok(data) = fs::read_to_string("custom_structures.json") {
            if let Ok(loaded_structures) = serde_json::from_str(&data) {
                self.custom_structures = loaded_structures;
            }
        }
    }

    fn save_custom_structures(&self) {
        if let Ok(json_data) = serde_json::to_string_pretty(&self.custom_structures) {
            fs::write("custom_structures.json", json_data).unwrap();
        }
    }

    fn render_setting_panel(&mut self, ui: &mut Ui) {
        ui.heading("Setting");

        self.render_custom_structures_list(ui);
        self.render_current_structure_panel(ui);
    }

    fn render_custom_structures_list(&mut self, ui: &mut Ui) {
        if ui.button("New Structure").clicked() {
            self.current_structure = CustomStructure::default();
        }

        let mut custom_structures = self.custom_structures.clone();

        ScrollArea::vertical().id_source("structure_list").show(ui, |ui| {
            let mut structures_to_remove = Vec::new();

            for (index, structure) in custom_structures.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button(&structure.name).clicked() {
                        self.current_structure = structure.clone();
                    }
                    if ui.button("❌").clicked() {
                        structures_to_remove.push(index);
                    }
                });
            }

            for index in structures_to_remove.iter().rev() {
                custom_structures.remove(*index);
            }
        });

        if custom_structures != self.custom_structures {
            self.custom_structures = custom_structures;
            self.save_custom_structures();
        }
    }

    fn render_current_structure_panel(&mut self, ui: &mut Ui) {
        ui.heading("Current Structure");

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.current_structure.name);
        });

        ui.horizontal(|ui| {
            if ui.button("Add Field").clicked() {
                self.current_structure.fields.push(String::new());
            }

            if ui.button("Save").clicked() {
                if let Some(index) = self
                    .custom_structures
                    .iter()
                    .position(|s| s.name == self.current_structure.name)
                {
                    self.custom_structures[index] = self.current_structure.clone();
                } else {
                    self.custom_structures.push(self.current_structure.clone());
                }
                self.save_custom_structures();
            }
        });

        ScrollArea::vertical().id_source("current_structure").show(ui, |ui| {
            let mut current_structure = self.current_structure.clone();

            for field in &mut current_structure.fields {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(field);
                    if ui.button("❌").clicked() {
                        let index = self
                            .current_structure
                            .fields
                            .iter()
                            .position(|f| f == field)
                            .unwrap();
                        self.current_structure.fields.remove(index);
                    }
                });
            }
        });
    }

    fn render_erp_panel(&mut self, ui: &mut Ui) {
        ui.heading("ERP");

        ScrollArea::vertical().id_source("erp_panel").show(ui, |ui| {
            for structure in &self.custom_structures {
                ui.horizontal(|ui| {
                    ui.label(&structure.name);
                    // Add ERP functionality here
                });
            }
        });
    }
}

impl eframe::App for ERPApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                if ui.button("Setting").clicked() {
                    self.show_setting_panel = true;
                }
            });

        if self.show_setting_panel {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.render_setting_panel(ui);
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.render_erp_panel(ui);
            });
        }
    }
}

fn main() {
    let mut app = ERPApp::default();
    app.load_custom_structures();

    let options = eframe::NativeOptions::default();
    eframe::run_native("ERP App", options, Box::new(|_cc| {
        Ok(Box::new(app))
    }));
}