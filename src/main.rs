use calamine::open_workbook;
use calamine::DataType;
use calamine::Reader;
use calamine::Xlsx;
use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ScrollArea, Ui, Vec2};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use xlsxwriter::Workbook;
// 데이터 저장을 위한 구조체 수정
#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
struct FieldValue {
    value: String,
    field_type: FieldType,
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct ERPData {
    structure_name: String,
    data: HashMap<String, Vec<HashMap<String, FieldValue>>>, // structure_name -> rows
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
enum FieldType {
    Text,
    Number,
    Date,
    Boolean,
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::Text
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
struct Field {
    name: String,
    field_type: FieldType,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
struct CustomStructure {
    name: String,
    fields: Vec<Field>,
}
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
struct CustomCategory {
    name: String,
    fields: Vec<CustomStructure>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
struct ERPApp {
    custom_structures: Vec<CustomCategory>,
    current_structure: CustomStructure,
    show_setting_panel: bool,
    erp_data: ERPData,
    show_structure_editor: bool, // 추가된 필드

    selected_structure: Option<String>,
    expanded_categories: HashMap<String, bool>,
    selected_category: Option<String>, // 현재 선택된 카테고리
}

impl ERPApp {
    // ERP 데이터 저장/로드 함수

    fn load_erp_data(&mut self) {
        if let Ok(data) = fs::read_to_string("erp_data.json") {
            if let Ok(loaded_data) = serde_json::from_str(&data) {
                self.erp_data = loaded_data;
            }
        }
    }

    fn save_erp_data(&self) {
        if let Ok(json_data) = serde_json::to_string_pretty(&self.erp_data) {
            fs::write("erp_data.json", json_data).unwrap();
        }
    }

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

    fn new() -> Self {
        let mut app = Self::default();
        app.load_custom_structures();
        app
    }

    fn render_setting_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("카테고리 관리");
            if ui.button("➕ 새 카테고리").clicked() {
                self.custom_structures.push(CustomCategory {
                    name: "새 카테고리".to_string(),
                    fields: Vec::new(),
                });
            }
            if ui.button("💾 저장하기").clicked() {
                self.save_custom_structures();
            }
        });
        ui.separator();

        // 카테고리 목록과 구조체 생성 버튼
        let mut category_to_remove = None;
        for (idx, category) in self.custom_structures.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut category.name);
                    if ui.button("🗑️").clicked() {
                        category_to_remove = Some(idx);
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("➕ 새 구조체 추가").clicked() {
                        self.current_structure = CustomStructure::default();
                        self.selected_category = Some(category.name.clone());
                        self.show_structure_editor = true;
                    }
                });

                // 해당 카테고리의 구조체 목록 표시
                for structure in &category.fields {
                    ui.horizontal(|ui| {
                        ui.label(&structure.name);
                        if ui.button("✏️").clicked() {
                            self.current_structure = structure.clone();
                            self.selected_category = Some(category.name.clone());
                            self.show_structure_editor = true;
                        }
                    });
                }
            });
        }

        // 카테고리 삭제 처리
        if let Some(idx) = category_to_remove {
            self.custom_structures.remove(idx);
        }

        // 구조체 편집기가 활성화된 경우에만 표시
        if self.show_structure_editor {
            self.render_structure_editor(ui);
        }
    }

    fn render_structure_editor(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("구조체 편집");

            ui.horizontal(|ui| {
                ui.label("구조체 이름:");
                ui.text_edit_singleline(&mut self.current_structure.name);
            });

            // 필드 관리
            ui.horizontal(|ui| {
                ui.heading("필드 목록");
                if ui.button("➕ 필드 추가").clicked() {
                    self.current_structure.fields.push(Field::default());
                }
            });

            // 필드 목록 표시
            let mut fields_to_remove = Vec::new();
            for (idx, field) in self.current_structure.fields.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut field.name);
                    egui::ComboBox::from_id_source(format!("field_type_{}", idx))
                        .selected_text(format!("{:?}", field.field_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut field.field_type, FieldType::Text, "텍스트");
                            ui.selectable_value(&mut field.field_type, FieldType::Number, "숫자");
                            ui.selectable_value(&mut field.field_type, FieldType::Date, "날짜");
                            ui.selectable_value(
                                &mut field.field_type,
                                FieldType::Boolean,
                                "참/거짓",
                            );
                        });
                    if ui.button("🗑️").clicked() {
                        fields_to_remove.push(idx);
                    }
                });
            }

            // 필드 삭제 처리
            for idx in fields_to_remove.iter().rev() {
                self.current_structure.fields.remove(*idx);
            }

            // 저장 버튼
            ui.horizontal(|ui| {
                if ui.button("💾 구조체 저장").clicked() {
                    if let Some(category_name) = &self.selected_category {
                        if let Some(category) = self
                            .custom_structures
                            .iter_mut()
                            .find(|c| &c.name == category_name)
                        {
                            // 기존 구조체 수정 또는 새 구조체 추가
                            if let Some(existing_idx) = category
                                .fields
                                .iter()
                                .position(|s| s.name == self.current_structure.name)
                            {
                                category.fields[existing_idx] = self.current_structure.clone();
                            } else {
                                category.fields.push(self.current_structure.clone());
                            }
                            self.save_custom_structures();
                            self.show_structure_editor = false;
                        }
                    }
                }
                if ui.button("❌ 취소").clicked() {
                    self.show_structure_editor = false;
                }
            });
        });
    }

    fn render_custom_structures_list(&mut self, ui: &mut Ui) {
        let mut custom_structures = self.custom_structures.clone();

        ScrollArea::vertical()
            .id_source("structure_list")
            .show(ui, |ui| {
                let mut structures_to_remove = Vec::new();

                // for (index, structure) in custom_structures.iter_mut().enumerate() {
                //     ui.horizontal(|ui| {
                //         if ui.button(&structure.name).clicked() {
                //             self.current_structure = structure.clone();
                //         }
                //         if ui.button("🗑️").clicked() {
                //             structures_to_remove.push(index);
                //         }
                //     });
                // }

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
        ui.heading("구조체 편집");
        ui.horizontal(|ui| {
            ui.label("카테고리:");
            let current_category = self
                .get_structure_category(&self.current_structure.name)
                .map(|s| s.to_string());

            egui::ComboBox::from_id_source("category_selector")
                .selected_text(current_category.as_deref().unwrap_or("선택 안됨"))
                .show_ui(ui, |ui| {
                    // let mut selected_category = None;

                    // for category in &self.categories {
                    //     if ui
                    //         .selectable_label(
                    //             current_category
                    //                 .as_ref()
                    //                 .map(|c| c == &category.name)
                    //                 .unwrap_or(false),
                    //             &category.name,
                    //         )
                    //         .clicked()
                    //     {
                    //         selected_category = Some(category.name.clone());
                    //     }
                    // }

                    // 선택된 카테고리가 있으면 처리
                    // if let Some(new_category) = selected_category {
                    //     // 기존 카테고리에서 제거
                    //     for cat in &mut self.categories {
                    //         cat.structures.retain(|s| s != &self.current_structure.name);
                    //     }
                    //     // 새 카테고리에 추가
                    //     if let Some(cat) =
                    //         self.categories.iter_mut().find(|c| c.name == new_category)
                    //     {
                    //         cat.structures.push(self.current_structure.name.clone());
                    //     }
                    //     self.save_categories();
                    // }
                });
        });

        ui.horizontal(|ui| {
            ui.label("이름:");
            ui.text_edit_singleline(&mut self.current_structure.name);
        });

        ui.horizontal(|ui| {
            if ui.button("➕ 필드 추가").clicked() {
                self.current_structure.fields.push(Field::default());
            }

            // if ui.button("💾 저장").clicked() && !self.current_structure.name.is_empty() {
            //     if let Some(index) = self
            //         .custom_structures
            //         .iter()
            //         .position(|s| s.name == self.current_structure.name)
            //     {
            //         self.custom_structures[index] = self.current_structure.clone();
            //     } else {
            //         self.custom_structures.push(self.current_structure.clone());
            //     }
            //     self.save_custom_structures();
            // }
        });

        ui.add_space(10.0);
        ui.heading("필드 목록");

        ScrollArea::vertical()
            .id_source("current_structure")
            .show(ui, |ui| {
                let mut fields_to_remove = Vec::new();

                for (index, field) in self.current_structure.fields.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label("이름:");
                        ui.text_edit_singleline(&mut field.name);

                        ui.label("타입:");
                        egui::ComboBox::from_id_source(format!("type_selector_{}", index))
                            .selected_text(format!("{:?}", field.field_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut field.field_type, FieldType::Text, "Text");
                                ui.selectable_value(
                                    &mut field.field_type,
                                    FieldType::Number,
                                    "Number",
                                );
                                ui.selectable_value(&mut field.field_type, FieldType::Date, "Date");
                                ui.selectable_value(
                                    &mut field.field_type,
                                    FieldType::Boolean,
                                    "Boolean",
                                );
                            });

                        if ui.button("🗑️").clicked() {
                            fields_to_remove.push(index);
                        }
                    });
                }

                for index in fields_to_remove.iter().rev() {
                    self.current_structure.fields.remove(*index);
                }
            });
    }

    fn render_erp_panel(&mut self, ui: &mut Ui) {
        if let Some(selected_structure_name) = &self.selected_structure.clone() {
            let selected_structure = self
                .custom_structures
                .iter()
                .find_map(|category| {
                    category
                        .fields
                        .iter()
                        .find(|structure| &structure.name == selected_structure_name)
                })
                .cloned();

            if let Some(structure) = selected_structure.clone() {
                // 상단 툴바
                ui.horizontal(|ui| {
                    ui.heading(&structure.name);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📥 CSV 내보내기").clicked() {
                            if let Err(e) = self.save_as_csv(&structure) {
                                eprintln!("CSV 내보내기 실패: {}", e);
                            }
                        }

                        if ui.button("📤 CSV 불러오기").clicked() {
                            if let Err(e) = self.load_from_csv(&structure) {
                                eprintln!("CSV 불러오기 실패: {}", e);
                            }
                        }

                        if ui.button("➕ 새 데이터").clicked() {
                            let mut new_row = HashMap::new();
                            for field in &structure.fields {
                                new_row.insert(
                                    field.name.clone(),
                                    FieldValue {
                                        value: String::new(),
                                        field_type: field.field_type.clone(),
                                    },
                                );
                            }

                            self.erp_data
                                .data
                                .entry(structure.name.clone())
                                .or_default()
                                .push(new_row);

                            self.save_erp_data();
                        }
                    });
                });
                ui.separator();

                // 테이블 그리기
                ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("erp_table")
                        .striped(true)
                        .spacing(Vec2::new(10.0, 4.0))
                        .show(ui, |ui| {
                            // 헤더 행
                            ui.label("No.");
                            for field in &structure.fields {
                                ui.label(&field.name);
                            }
                            ui.label("관리");
                            ui.end_row();

                            let structure_name = structure.name.clone();
                            let mut rows_data = self
                                .erp_data
                                .data
                                .get_mut(&structure_name)
                                .cloned()
                                .unwrap_or_default();

                            let mut row_to_remove = None;

                            for (row_idx, row_data) in rows_data.iter_mut().enumerate() {
                                ui.label((row_idx + 1).to_string());

                                for field in &structure.fields {
                                    let field_value = row_data
                                        .entry(field.name.clone())
                                        .or_insert_with(|| FieldValue {
                                            value: String::new(),
                                            field_type: field.field_type.clone(),
                                        });

                                    match field_value.field_type {
                                        FieldType::Text => {
                                            let mut value = field_value.value.clone();
                                            if ui.text_edit_singleline(&mut value).changed() {
                                                field_value.value = value;
                                            }
                                        }
                                        FieldType::Number => {
                                            let mut value =
                                                field_value.value.parse::<f64>().unwrap_or(0.0);
                                            if ui.add(egui::DragValue::new(&mut value)).changed() {
                                                field_value.value = value.to_string();
                                            }
                                        }
                                        FieldType::Date => {
                                            let mut value = field_value.value.clone();
                                            if ui.text_edit_singleline(&mut value).changed() {
                                                field_value.value = value;
                                            }
                                        }
                                        FieldType::Boolean => {
                                            let mut value = field_value.value == "true";
                                            if ui.checkbox(&mut value, "").changed() {
                                                field_value.value = value.to_string();
                                            }
                                        }
                                    }
                                }

                                if ui.button("🗑️").clicked() {
                                    row_to_remove = Some(row_idx);
                                }

                                ui.end_row();
                            }

                            // 모든 변경사항을 한 번에 처리
                            if let Some(idx) = row_to_remove {
                                rows_data.remove(idx);
                            }

                            // 데이터를 한 번에 업데이트하고 저장
                            if self.erp_data.data.get(&structure_name) != Some(&rows_data) {
                                self.erp_data.data.insert(structure_name, rows_data);
                                self.save_erp_data();
                            }
                        });
                });
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("왼쪽 메뉴에서 구조체를 선택해주세요");
            });
        }
    }
    fn save_as_csv(&self, structure: &CustomStructure) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(format!("{}.csv", structure.name))?;

        // Write headers
        let headers: Vec<String> = structure
            .fields
            .iter()
            .map(|field| field.name.clone())
            .collect();
        wtr.write_record(&headers)?;

        // Write data
        if let Some(rows) = self.erp_data.data.get(&structure.name) {
            for row_data in rows {
                let record: Vec<String> = structure
                    .fields
                    .iter()
                    .map(|field| {
                        row_data
                            .get(&field.name)
                            .map(|fv| fv.value.clone())
                            .unwrap_or_default()
                    })
                    .collect();
                wtr.write_record(&record)?;
            }
        }

        wtr.flush()?;
        Ok(())
    }

    fn load_from_csv(
        &mut self,
        structure: &CustomStructure,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = csv::Reader::from_path(format!("{}.csv", structure.name))?;
        let mut new_rows = Vec::new();

        for result in rdr.records() {
            let record = result?;
            let mut row_data = HashMap::new();

            for (idx, field) in structure.fields.iter().enumerate() {
                let value = record.get(idx).unwrap_or_default().to_string();
                row_data.insert(
                    field.name.clone(),
                    FieldValue {
                        value,
                        field_type: field.field_type.clone(),
                    },
                );
            }

            new_rows.push(row_data);
        }

        self.erp_data.data.insert(structure.name.clone(), new_rows);
        self.save_erp_data();
        Ok(())
    }
    fn render_sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.heading("ERP 시스템");
        ui.separator();

        ScrollArea::vertical()
            .id_source("sidebar_menu")
            .show(ui, |ui| {
                // 카테고리별로 구조체 표시
                for category in &self.custom_structures {
                    let is_expanded = self
                        .expanded_categories
                        .entry(category.name.clone())
                        .or_insert(true);

                    ui.horizontal(|ui| {
                        if ui.button(if *is_expanded { "📂" } else { "📁" }).clicked() {
                            *is_expanded = !*is_expanded;
                        }
                        ui.label(&category.name);
                    });

                    if *is_expanded {
                        ui.indent(category.name.clone(), |ui| {
                            for structure in &category.fields {
                                let selected = self
                                    .selected_structure
                                    .as_ref()
                                    .map_or(false, |s| s == &structure.name);

                                if ui.selectable_label(selected, &structure.name).clicked() {
                                    self.selected_structure = Some(structure.name.clone());
                                    self.show_setting_panel = false;
                                }
                            }
                        });
                    }
                }
            });

        // 설정 버튼
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(10.0);
            let setting_btn = egui::Button::new("⚙️")
                .min_size(Vec2::new(30.0, 30.0))
                .frame(false);
            if ui.add(setting_btn).clicked() {
                self.show_setting_panel = !self.show_setting_panel;
            }
            ui.add_space(10.0);
            ui.separator();
        });
    }
    fn is_structure_in_any_category(&self, structure_name: &str) -> bool {
        false
        // self.categories
        //     .iter()
        //     .any(|cat| cat.structures.contains(&structure_name.to_string()))
    }

    fn get_structure_category(&self, structure_name: &str) -> Option<&String> {
        for category in &self.custom_structures {
            // if category.structures.contains(&structure_name.to_string()) {
            //     return Some(&category.name);
            // }
        }
        None
    }
}

impl eframe::App for ERPApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 폰트 설정
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "nanum_gothic".to_owned(),
            FontData::from_static(include_bytes!("../assets/fonts/NanumGothic-Bold.ttf")),
        );

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "nanum_gothic".to_owned());

        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "nanum_gothic".to_owned());

        ctx.set_fonts(fonts);

        // 사이드바 구현
        egui::SidePanel::left("side_panel")
            .max_width(200.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_setting_panel {
                self.render_setting_panel(ui);
            } else {
                self.render_erp_panel(ui);
            }
        });
    }
}

fn main() {
    let mut app = ERPApp::default();
    app.load_custom_structures();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([980.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native("Ruquest", options, Box::new(|_cc| Ok(Box::new(app)))).unwrap();
}
