use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ScrollArea, Ui, Vec2};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;

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
#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
struct Category {
    name: String,
    structures: Vec<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
struct Field {
    name: String,
    field_type: FieldType,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
struct CustomStructure {
    name: String,
    fields: Vec<Field>,
}

#[derive(Default)]
struct ERPApp {
    custom_structures: Vec<CustomStructure>,
    current_structure: CustomStructure,
    show_setting_panel: bool,
    erp_data: ERPData,
    selected_structure: Option<String>,
    categories: Vec<Category>,                  // 카테고리 목록 추가
    expanded_categories: HashMap<String, bool>, // 카테고리 확장 상태 저장
}

impl ERPApp {
    fn new() -> Self {
        let mut app = Self::default();
        app.load_categories();
        app.load_custom_structures();
        app
    }
    // ERP 데이터 저장/로드 함수
    fn save_erp_data(&self) {
        if let Ok(json_data) = serde_json::to_string_pretty(&self.erp_data) {
            fs::write("erp_data.json", json_data).unwrap();
        }
    }

    fn load_erp_data(&mut self) {
        if let Ok(data) = fs::read_to_string("erp_data.json") {
            if let Ok(loaded_data) = serde_json::from_str(&data) {
                self.erp_data = loaded_data;
            }
        }
    }
    fn save_categories(&self) {
        if let Ok(json_data) = serde_json::to_string_pretty(&self.categories) {
            fs::write("categories.json", json_data).unwrap();
        }
    }

    fn load_categories(&mut self) {
        if let Ok(data) = fs::read_to_string("categories.json") {
            if let Ok(loaded_categories) = serde_json::from_str(&data) {
                self.categories = loaded_categories;
            }
        }
    }

    fn setup_custom_fonts(ctx: &Context) {
        // 폰트 정의 생성
        let mut fonts = FontDefinitions::default();

        // 나눔고딕 폰트 데이터 추가 (바이트 데이터로)
        fonts.font_data.insert(
            "nanum_gothic".to_owned(),
            FontData::from_static(include_bytes!("../assets/fonts/NanumGothic-Bold.ttf")),
        );

        // 프로포셔널 폰트 패밀리에 나눔고딕 추가
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "nanum_gothic".to_owned());

        // 고정폭 폰트 패밀리에도 나눔고딕 추가
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "nanum_gothic".to_owned());

        // 폰트 적용
        ctx.set_fonts(fonts);
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

    fn render_setting_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("카테고리 관리");
            if ui.button("➕ 새 카테고리").clicked() {
                self.categories.push(Category {
                    name: "새 카테고리".to_string(),
                    structures: Vec::new(),
                });
                self.save_categories();
            }
        });
        ui.separator();

        // 카테고리 목록
        let mut category_to_remove = None;

        for (idx, category) in self.categories.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut category.name);
                if ui.button("🗑️").clicked() {
                    category_to_remove = Some(idx);
                }
            });
        }

        // 삭제 처리를 반복문 밖에서 수행
        if let Some(idx) = category_to_remove {
            self.categories.remove(idx);
            self.save_categories();
        }

        ui.add_space(20.0);
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.heading("구조체 설정");
        });
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(20.0);

        ui.columns(2, |columns| {
            // 왼쪽 카드: 구조체 목록
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(245, 245, 245))
                .rounding(8.0)
                .inner_margin(Vec2::new(10.0, 10.0))
                .show(&mut columns[0], |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("구조체 목록");
                    });
                    ui.add_space(10.0);

                    let button_width = ui.available_width();
                    if ui
                        .add_sized(
                            Vec2::new(button_width, 30.0),
                            egui::Button::new("➕ 새 구조체 추가"),
                        )
                        .clicked()
                    {
                        self.current_structure = CustomStructure::default();
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ScrollArea::vertical()
                        .id_source("structure_list")
                        .show(ui, |ui| {
                            let mut structures_to_remove = Vec::new();

                            for (index, structure) in self.custom_structures.iter().enumerate() {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    let selected = self.current_structure.name == structure.name;
                                    let label_width = ui.available_width() - 30.0;
                                    if ui
                                        .add_sized(
                                            Vec2::new(label_width, 24.0),
                                            egui::SelectableLabel::new(selected, &structure.name),
                                        )
                                        .clicked()
                                    {
                                        self.current_structure = structure.clone();
                                    }
                                    if ui.small_button("🗑️").clicked() {
                                        structures_to_remove.push(index);
                                    }
                                });
                            }

                            for index in structures_to_remove.iter().rev() {
                                self.custom_structures.remove(*index);
                            }
                        });
                });

            // 오른쪽 카드: 구조체 편집
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(245, 245, 245))
                .rounding(8.0)
                .inner_margin(Vec2::new(10.0, 10.0))
                .show(&mut columns[1], |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("구조체 편집");
                    });
                    ui.add_space(10.0);

                    // 구조체 이름 입력
                    ui.horizontal(|ui| {
                        ui.label("구조체 이름:");
                        ui.add_sized(
                            Vec2::new(ui.available_width(), 30.0),
                            egui::TextEdit::singleline(&mut self.current_structure.name)
                                .hint_text("구조체 이름을 입력하세요"),
                        );
                    });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // 필드 관리 섹션
                    ui.horizontal(|ui| {
                        ui.heading("필드 목록");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("➕ 필드 추가").clicked() {
                                self.current_structure.fields.push(Field::default());
                            }
                        });
                    });
                    ui.add_space(10.0);

                    ScrollArea::vertical()
                        .id_source("fields_list")
                        .show(ui, |ui| {
                            let mut fields_to_remove = Vec::new();

                            for (index, field) in
                                self.current_structure.fields.iter_mut().enumerate()
                            {
                                ui.add_space(4.0);
                                egui::Frame::default()
                                    .fill(egui::Color32::from_rgb(255, 255, 255))
                                    .rounding(4.0)
                                    .inner_margin(Vec2::new(8.0, 8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add_sized(
                                                Vec2::new(ui.available_width() * 0.4, 24.0),
                                                egui::TextEdit::singleline(&mut field.name)
                                                    .hint_text("필드 이름"),
                                            );

                                            egui::ComboBox::from_id_source(format!(
                                                "type_selector_{}",
                                                index
                                            ))
                                            .selected_text(format!("{:?}", field.field_type))
                                            .width(120.0)
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    ui.selectable_value(
                                                        &mut field.field_type,
                                                        FieldType::Text,
                                                        "텍스트",
                                                    );
                                                    ui.selectable_value(
                                                        &mut field.field_type,
                                                        FieldType::Number,
                                                        "숫자",
                                                    );
                                                    ui.selectable_value(
                                                        &mut field.field_type,
                                                        FieldType::Date,
                                                        "날짜",
                                                    );
                                                    ui.selectable_value(
                                                        &mut field.field_type,
                                                        FieldType::Boolean,
                                                        "참/거짓",
                                                    );
                                                },
                                            );

                                            if ui.small_button("🗑️").clicked() {
                                                fields_to_remove.push(index);
                                            }
                                        });
                                    });
                            }

                            for index in fields_to_remove.iter().rev() {
                                self.current_structure.fields.remove(*index);
                            }
                        });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // 저장 버튼
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        let save_button =
                            egui::Button::new(egui::RichText::new("💾 저장").size(16.0))
                                .min_size(Vec2::new(100.0, 35.0))
                                .fill(egui::Color32::from_rgb(100, 185, 255));

                        if ui
                            .add_enabled(!self.current_structure.name.is_empty(), save_button)
                            .clicked()
                        {
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
                });
        });
    }

    fn render_custom_structures_list(&mut self, ui: &mut Ui) {
        let mut custom_structures = self.custom_structures.clone();

        ScrollArea::vertical()
            .id_source("structure_list")
            .show(ui, |ui| {
                let mut structures_to_remove = Vec::new();

                for (index, structure) in custom_structures.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.button(&structure.name).clicked() {
                            self.current_structure = structure.clone();
                        }
                        if ui.button("🗑️").clicked() {
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
        ui.heading("구조체 편집");
        ui.horizontal(|ui| {
            ui.label("카테고리:");
            let current_category = self
                .get_structure_category(&self.current_structure.name)
                .map(|s| s.to_string());

            egui::ComboBox::from_id_source("category_selector")
                .selected_text(current_category.as_deref().unwrap_or("선택 안됨"))
                .show_ui(ui, |ui| {
                    let mut selected_category = None;

                    for category in &self.categories {
                        if ui
                            .selectable_label(
                                current_category
                                    .as_ref()
                                    .map(|c| c == &category.name)
                                    .unwrap_or(false),
                                &category.name,
                            )
                            .clicked()
                        {
                            selected_category = Some(category.name.clone());
                        }
                    }

                    // 선택된 카테고리가 있으면 처리
                    if let Some(new_category) = selected_category {
                        // 기존 카테고리에서 제거
                        for cat in &mut self.categories {
                            cat.structures.retain(|s| s != &self.current_structure.name);
                        }
                        // 새 카테고리에 추가
                        if let Some(cat) =
                            self.categories.iter_mut().find(|c| c.name == new_category)
                        {
                            cat.structures.push(self.current_structure.name.clone());
                        }
                        self.save_categories();
                    }
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

            if ui.button("💾 저장").clicked() && !self.current_structure.name.is_empty() {
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
        if let Some(selected) = &self.selected_structure {
            let structure = self
                .custom_structures
                .iter()
                .find(|s| &s.name == selected)
                .cloned();

            if let Some(structure) = structure {
                // 상단 툴바
                ui.horizontal(|ui| {
                    ui.heading(&structure.name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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

                            let mut any_data_changed = false;
                            let mut rows_to_remove = None;
                            let structure_name = structure.name.clone();

                            // 데이터 행들
                            if let Some(rows) = self.erp_data.data.get_mut(&structure_name) {
                                for (row_idx, row_data) in rows.iter_mut().enumerate() {
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
                                                    any_data_changed = true;
                                                }
                                            }
                                            FieldType::Number => {
                                                let mut value =
                                                    field_value.value.parse::<f64>().unwrap_or(0.0);
                                                if ui
                                                    .add(egui::DragValue::new(&mut value))
                                                    .changed()
                                                {
                                                    field_value.value = value.to_string();
                                                    any_data_changed = true;
                                                }
                                            }
                                            FieldType::Date => {
                                                let mut value = field_value.value.clone();
                                                if ui.text_edit_singleline(&mut value).changed() {
                                                    field_value.value = value;
                                                    any_data_changed = true;
                                                }
                                            }
                                            FieldType::Boolean => {
                                                let mut value = field_value.value == "true";
                                                if ui.checkbox(&mut value, "").changed() {
                                                    field_value.value = value.to_string();
                                                    any_data_changed = true;
                                                }
                                            }
                                        }
                                    }

                                    // 삭제 버튼
                                    if ui.button("🗑️").clicked() {
                                        rows_to_remove = Some(row_idx);
                                    }

                                    ui.end_row();
                                }
                            }

                            // 데이터 변경사항 저장
                            if any_data_changed {
                                self.save_erp_data();
                            }

                            // 행 삭제 처리
                            if let Some(row_idx) = rows_to_remove {
                                if let Some(rows) = self.erp_data.data.get_mut(&structure_name) {
                                    rows.remove(row_idx);
                                    self.save_erp_data();
                                }
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

    fn render_sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.heading("ERP 시스템");
        ui.separator();

        ScrollArea::vertical()
            .id_source("sidebar_menu")
            .show(ui, |ui| {
                // 카테고리별로 구조체 표시
                for category in &self.categories {
                    let is_expanded = self
                        .expanded_categories
                        .entry(category.name.clone())
                        .or_insert(true); // 기본값을 true로 변경

                    ui.horizontal(|ui| {
                        if ui.button(if *is_expanded { "📂" } else { "📁" }).clicked() {
                            *is_expanded = !*is_expanded;
                        }
                        ui.label(&category.name);
                    });

                    if *is_expanded {
                        ui.indent(category.name.clone(), |ui| {
                            for structure_name in &category.structures {
                                if let Some(_) = self
                                    .custom_structures
                                    .iter()
                                    .find(|s| &s.name == structure_name)
                                {
                                    let selected =
                                        self.selected_structure.as_ref() == Some(structure_name);
                                    if ui.selectable_label(selected, structure_name).clicked() {
                                        self.selected_structure = Some(structure_name.clone());
                                        self.show_setting_panel = false;
                                    }
                                }
                            }
                        });
                    }
                }

                // 미분류 구조체 표시
                let uncategorized: Vec<_> = self
                    .custom_structures
                    .iter()
                    .filter(|structure| !self.is_structure_in_any_category(&structure.name))
                    .collect();

                if !uncategorized.is_empty() {
                    ui.separator();
                    ui.label("미분류");
                    ui.indent("uncategorized", |ui| {
                        for structure in uncategorized {
                            let selected =
                                self.selected_structure.as_ref() == Some(&structure.name);
                            if ui.selectable_label(selected, &structure.name).clicked() {
                                self.selected_structure = Some(structure.name.clone());
                                self.show_setting_panel = false;
                            }
                        }
                    });
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
        self.categories
            .iter()
            .any(|cat| cat.structures.contains(&structure_name.to_string()))
    }

    fn get_structure_category(&self, structure_name: &str) -> Option<&String> {
        for category in &self.categories {
            if category.structures.contains(&structure_name.to_string()) {
                return Some(&category.name);
            }
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
                ui.add_space(10.0);
                ui.heading("ERP 시스템");
                ui.separator();

                ScrollArea::vertical()
                    .id_source("sidebar_menu")
                    .show(ui, |ui| {
                        for structure in &self.custom_structures {
                            let selected =
                                self.selected_structure.as_ref() == Some(&structure.name);
                            let label_width = ui.available_width();
                            if ui
                                .add_sized(
                                    Vec2::new(label_width, 24.0),
                                    egui::SelectableLabel::new(selected, &structure.name),
                                )
                                .clicked()
                            {
                                self.selected_structure = Some(structure.name.clone());
                                self.show_setting_panel = false;
                            }
                        }
                    });

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
