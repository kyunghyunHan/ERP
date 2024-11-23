use calamine::open_workbook;
use calamine::DataType;
use calamine::Reader;
use calamine::Xlsx;
use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ScrollArea, Ui, Vec2};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::error::Error;
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
struct SubCategory {
    name: String,
    structures: Vec<CustomStructure>,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
struct CustomCategory {
    name: String,
    subcategories: Vec<SubCategory>,
}

#[derive(Default)]
struct ERPApp {
    custom_structures: Vec<CustomCategory>,
    current_structure: CustomStructure,
    current_subcategory: Option<String>, // 현재 선택된 서브카테고리
    show_setting_panel: bool,
    show_structure_editor: bool,
    erp_data: ERPData,
    selected_structure: Option<String>,
    selected_category: Option<String>,
    expanded_categories: HashMap<String, bool>,
    expanded_subcategories: HashMap<String, bool>, // 서브카테고리 확장 상태
}

impl ERPApp {
    fn find_structure(&self, structure_name: &str) -> Option<CustomStructure> {
        for category in &self.custom_structures {
            for subcategory in &category.subcategories {
                if let Some(structure) = subcategory
                    .structures
                    .iter()
                    .find(|s| s.name == structure_name)
                {
                    return Some(structure.clone());
                }
            }
        }
        None
    }

    fn load_structure_data(&mut self, structure_name: &str) {
        if let Ok(mut rdr) = csv::Reader::from_path(format!("{}.csv", structure_name)) {
            let mut rows = Vec::new();

            for result in rdr.records() {
                if let Ok(record) = result {
                    if let Some(structure) = self.find_structure(structure_name) {
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

                        rows.push(row_data);
                    }
                }
            }

            self.erp_data.data.insert(structure_name.to_string(), rows);
            self.save_erp_data();
        } else {
            self.erp_data
                .data
                .insert(structure_name.to_string(), Vec::new());
            self.save_erp_data();
        }
    }
    fn load_erp_data(&mut self) {
        match fs::read_to_string("erp_data.json") {
            Ok(data) => {
                match serde_json::from_str(&data) {
                    Ok(loaded_data) => {
                        self.erp_data = loaded_data;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse ERP data: {}", e);
                        // 파일이 손상된 경우 새로운 데이터로 초기화
                        self.erp_data = ERPData::default();
                    }
                }
            }
            Err(_) => {
                // 파일이 없는 경우 새로운 데이터로 초기화
                self.erp_data = ERPData::default();
                self.save_erp_data(); // 빈 데이터 파일 생성
            }
        }
    }
    // Excel 내보내기 (파일 선택 대화상자 사용)
    fn export_to_excel(
        &self,
        structure: &CustomStructure,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Excel Files", &["xlsx"])
            .set_file_name(&format!("{}.xlsx", structure.name))
            .save_file()
        {
            let workbook = Workbook::new(path.to_str().unwrap())?;
            let mut sheet = workbook.add_worksheet(None)?;

            // 헤더 작성
            for (col, field) in structure.fields.iter().enumerate() {
                sheet.write_string(0, col as u16, &field.name, None)?;
            }

            // 데이터 작성
            if let Some(rows) = self.erp_data.data.get(&structure.name) {
                for (row_idx, row_data) in rows.iter().enumerate() {
                    for (col, field) in structure.fields.iter().enumerate() {
                        if let Some(field_value) = row_data.get(&field.name) {
                            match field_value.field_type {
                                FieldType::Number => {
                                    if let Ok(num) = field_value.value.parse::<f64>() {
                                        sheet.write_number(
                                            row_idx as u32 + 1,
                                            col as u16,
                                            num,
                                            None,
                                        )?;
                                    } else {
                                        sheet.write_string(
                                            row_idx as u32 + 1,
                                            col as u16,
                                            &field_value.value,
                                            None,
                                        )?;
                                    }
                                }
                                FieldType::Boolean => {
                                    if let Ok(bool_val) = field_value.value.parse::<bool>() {
                                        sheet.write_boolean(
                                            row_idx as u32 + 1,
                                            col as u16,
                                            bool_val,
                                            None,
                                        )?;
                                    } else {
                                        sheet.write_string(
                                            row_idx as u32 + 1,
                                            col as u16,
                                            &field_value.value,
                                            None,
                                        )?;
                                    }
                                }
                                _ => {
                                    sheet.write_string(
                                        row_idx as u32 + 1,
                                        col as u16,
                                        &field_value.value,
                                        None,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }

            workbook.close()?;
            println!("Excel 파일 저장 완료!");
        }
        Ok(())
    }
    fn import_from_excel(&mut self, structure: &CustomStructure) -> Result<(), Box<dyn Error>> {
        // 파일 선택 대화상자
        if let Some(path) = FileDialog::new()
            .add_filter("Excel Files", &["xlsx"])
            .pick_file()
        {
            let mut workbook: Xlsx<_> = open_workbook(path)?;
            let range = match workbook.worksheet_range_at(0) {
                Some(Ok(range)) => range,
                Some(Err(e)) => return Err(e.into()),
                None => return Err("시트가 비어있습니다".into()),
            };

            let mut rows = Vec::new();
            for row_idx in 1..range.height() {
                let mut row_data = HashMap::new();
                for (col_idx, field) in structure.fields.iter().enumerate() {
                    let value = match range.get_value((row_idx as u32, col_idx as u32)) {
                        Some(DataType::Int(i)) => i.to_string(),
                        Some(DataType::Float(f)) => f.to_string(),
                        Some(DataType::String(s)) => s.to_string(),
                        Some(DataType::Bool(b)) => b.to_string(),
                        _ => String::new(),
                    };
                    row_data.insert(
                        field.name.clone(),
                        FieldValue {
                            value,
                            field_type: field.field_type.clone(),
                        },
                    );
                }
                rows.push(row_data);
            }

            // 데이터 저장 및 CSV 자동 백업
            self.erp_data.data.insert(structure.name.clone(), rows);
            self.save_to_csv(&structure.name);
            println!("Excel 파일 불러오기 완료!");
        }
        Ok(())
    }
    fn save_erp_data(&self) {
        if let Ok(json_data) = serde_json::to_string_pretty(&self.erp_data) {
            if let Err(e) = fs::write("erp_data.json", json_data) {
                eprintln!("Failed to save ERP data: {}", e);
            }
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
        println!("Saving structures to file...");
        if let Ok(json_data) = serde_json::to_string_pretty(&self.custom_structures) {
            if let Err(e) = fs::write("custom_structures.json", json_data) {
                println!("Failed to save structures: {}", e);
            } else {
                println!("Structures saved successfully");
            }
        } else {
            println!("Failed to serialize structures");
        }
    }

    fn new() -> Self {
        let mut app = Self::default();
        app.load_custom_structures();
        app.load_erp_data(); // 시작할 때 ERP 데이터도 로드
        app
    }
    fn render_setting_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("카테고리 관리");
            if ui.button("➕ 새 카테고리").clicked() {
                self.custom_structures.push(CustomCategory {
                    name: "새 카테고리".to_string(),
                    subcategories: Vec::new(),
                });
            }
            if ui.button("💾 저장하기").clicked() {
                self.save_custom_structures();
            }
        });
        ui.separator();

        // 카테고리 목록
        let mut category_to_remove = None;
        for (cat_idx, category) in self.custom_structures.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut category.name);
                    if ui.button("🗑️").clicked() {
                        category_to_remove = Some(cat_idx);
                    }
                    if ui.button("➕ 새 서브카테고리").clicked() {
                        category.subcategories.push(SubCategory {
                            name: "새 서브카테고리".to_string(),
                            structures: Vec::new(),
                        });
                    }
                });

                // 서브카테고리 목록
                let mut subcategory_to_remove = None;
                for (sub_idx, subcategory) in category.subcategories.iter_mut().enumerate() {
                    ui.indent(format!("sub_{}", sub_idx), |ui| {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut subcategory.name);
                            if ui.button("🗑️").clicked() {
                                subcategory_to_remove = Some(sub_idx);
                            }
                            if ui.button("➕ 새 구조체").clicked() {
                                self.current_structure = CustomStructure::default();
                                self.selected_category = Some(category.name.clone());
                                self.current_subcategory = Some(subcategory.name.clone());
                                self.show_structure_editor = true;
                            }
                        });

                        // 해당 서브카테고리의 구조체 목록
                        for structure in &subcategory.structures {
                            ui.horizontal(|ui| {
                                ui.label(&structure.name);
                                if ui.button("✏️").clicked() {
                                    self.current_structure = structure.clone();
                                    self.selected_category = Some(category.name.clone());
                                    self.current_subcategory = Some(subcategory.name.clone());
                                    self.show_structure_editor = true;
                                }
                            });
                        }
                    });
                }

                if let Some(idx) = subcategory_to_remove {
                    category.subcategories.remove(idx);
                }
            });
        }

        // 카테고리 삭제 처리
        if let Some(idx) = category_to_remove {
            self.custom_structures.remove(idx);
        }

        // 구조체 편집기
        if self.show_structure_editor {
            self.render_structure_editor(ui);
        }
    }
    fn save_to_csv(&self, structure_name: &str) {
        if let Some(structure) = self.find_structure(structure_name) {
            if let Some(rows) = self.erp_data.data.get(structure_name) {
                match csv::Writer::from_path(format!("{}.csv", structure_name)) {
                    Ok(mut writer) => {
                        // 헤더 작성
                        let headers: Vec<String> = structure
                            .fields
                            .iter()
                            .map(|field| field.name.clone())
                            .collect();
                        if let Err(e) = writer.write_record(&headers) {
                            eprintln!("헤더 저장 실패: {}", e);
                            return;
                        }

                        // 데이터 작성
                        for row in rows {
                            let record: Vec<String> = structure
                                .fields
                                .iter()
                                .map(|field| {
                                    row.get(&field.name)
                                        .map(|fv| fv.value.clone())
                                        .unwrap_or_default()
                                })
                                .collect();
                            if let Err(e) = writer.write_record(&record) {
                                eprintln!("데이터 저장 실패: {}", e);
                                return;
                            }
                        }

                        if let Err(e) = writer.flush() {
                            eprintln!("파일 저장 실패: {}", e);
                            return;
                        }
                        println!("CSV 파일 저장 완료: {}.csv", structure_name);
                    }
                    Err(e) => {
                        eprintln!("CSV 파일 생성 실패: {}", e);
                    }
                }
            }
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
            ScrollArea::vertical()
                .id_source("fields_list")
                .show(ui, |ui| {
                    for (idx, field) in self.current_structure.fields.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("필드 이름:");
                                ui.add_sized(
                                    Vec2::new(150.0, 20.0),
                                    egui::TextEdit::singleline(&mut field.name),
                                );

                                ui.label("타입:");
                                egui::ComboBox::from_id_source(format!("field_type_{}", idx))
                                    .selected_text(format!("{:?}", field.field_type))
                                    .show_ui(ui, |ui| {
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
                                    });

                                if ui.button("🗑️ 삭제").clicked() {
                                    fields_to_remove.push(idx);
                                }
                            });
                        });
                    }
                });

            // 필드 삭제 처리
            for idx in fields_to_remove.iter().rev() {
                self.current_structure.fields.remove(*idx);
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // 저장 버튼 섹션
            ui.horizontal(|ui| {
                if ui.button("💾 구조체 저장").clicked() {
                    if !self.current_structure.name.is_empty() {
                        let category_name = self.selected_category.clone();
                        let subcategory_name = self.current_subcategory.clone();
                        let structure_name = self.current_structure.name.clone();

                        if let (Some(cat_name), Some(subcat_name)) =
                            (category_name, subcategory_name)
                        {
                            if let Some(category) = self
                                .custom_structures
                                .iter_mut()
                                .find(|c| c.name == cat_name)
                            {
                                if let Some(subcategory) = category
                                    .subcategories
                                    .iter_mut()
                                    .find(|s| s.name == subcat_name)
                                {
                                    let is_new = !subcategory
                                        .structures
                                        .iter()
                                        .any(|s| s.name == self.current_structure.name);

                                    // 구조체 저장
                                    if let Some(idx) = subcategory
                                        .structures
                                        .iter()
                                        .position(|s| s.name == self.current_structure.name)
                                    {
                                        subcategory.structures[idx] =
                                            self.current_structure.clone();
                                    } else {
                                        subcategory.structures.push(self.current_structure.clone());
                                    }

                                    // 새 구조체인 경우 빈 데이터 초기화
                                    if is_new {
                                        self.erp_data.data.insert(structure_name, Vec::new());
                                        self.save_erp_data();
                                    }

                                    self.save_custom_structures();
                                    self.show_structure_editor = false;

                                    // 성공 메시지 출력
                                    println!("구조체가 성공적으로 저장되었습니다!");
                                } else {
                                    println!("서브카테고리를 찾을 수 없습니다: {}", subcat_name);
                                }
                            } else {
                                println!("카테고리를 찾을 수 없습니다: {}", cat_name);
                            }
                        } else {
                            println!("카테고리 또는 서브카테고리가 선택되지 않았습니다.");
                        }
                    } else {
                        println!("구조체 이름을 입력해주세요!");
                    }
                }

                if ui.button("❌ 취소").clicked() {
                    self.show_structure_editor = false;
                }

                // 현재 선택된 카테고리와 서브카테고리 정보 표시
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.label(format!(
                        "저장 위치: {} > {}",
                        self.selected_category
                            .as_ref()
                            .unwrap_or(&"없음".to_string()),
                        self.current_subcategory
                            .as_ref()
                            .unwrap_or(&"없음".to_string())
                    ));
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
            // 선택된 구조체 찾기
            let selected_structure = self
                .custom_structures
                .iter()
                .find_map(|category| {
                    category.subcategories.iter().find_map(|subcategory| {
                        subcategory
                            .structures
                            .iter()
                            .find(|structure| &structure.name == selected_structure_name)
                    })
                })
                .cloned();

            if let Some(structure) = selected_structure {
                // 상단 툴바
                ui.horizontal(|ui| {
                    ui.heading(&structure.name);
                    let structure_clone = structure.clone();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📥 Excel 내보내기").clicked() {
                            if let Err(e) = self.export_to_excel(&structure_clone) {
                                eprintln!("Excel 내보내기 실패: {}", e);
                            }
                        }

                        if ui.button("📤 Excel 불러오기").clicked() {
                            if let Err(e) = self.import_from_excel(&structure_clone) {
                                eprintln!("Excel 불러오기 실패: {}", e);
                            }
                        }

                        if ui.button("➕ 새 데이터").clicked() {
                            let mut new_row = HashMap::new();
                            for field in &structure_clone.fields {
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
                                .entry(structure_clone.name.clone())
                                .or_default()
                                .push(new_row);

                            self.save_to_csv(&structure_clone.name);
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

                            // 변경사항 처리
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
                // 먼저 필요한 데이터를 복사
                let categories_data: Vec<(&CustomCategory, bool)> = self
                    .custom_structures
                    .iter()
                    .map(|category| {
                        let is_expanded = *self
                            .expanded_categories
                            .get(&category.name)
                            .unwrap_or(&true);
                        (category, is_expanded)
                    })
                    .collect();

                // 상태 변경을 저장할 벡터들
                let mut toggle_category: Option<String> = None;
                let mut toggle_subcategory: Option<(String, String)> = None;
                let mut select_structure: Option<String> = None;

                // UI 렌더링
                for (category, is_category_expanded) in categories_data {
                    ui.horizontal(|ui| {
                        if ui
                            .button(if is_category_expanded { "📂" } else { "📁" })
                            .clicked()
                        {
                            toggle_category = Some(category.name.clone());
                        }
                        ui.label(&category.name);
                    });

                    if is_category_expanded {
                        ui.indent(category.name.clone(), |ui| {
                            for subcategory in &category.subcategories {
                                let sub_expanded = *self
                                    .expanded_subcategories
                                    .get(&format!("{}-{}", category.name, subcategory.name))
                                    .unwrap_or(&true);

                                ui.horizontal(|ui| {
                                    if ui.button(if sub_expanded { "📂" } else { "📁" }).clicked()
                                    {
                                        toggle_subcategory =
                                            Some((category.name.clone(), subcategory.name.clone()));
                                    }
                                    ui.label(&subcategory.name);
                                });

                                if sub_expanded {
                                    ui.indent(subcategory.name.clone(), |ui| {
                                        for structure in &subcategory.structures {
                                            let selected = self
                                                .selected_structure
                                                .as_ref()
                                                .map_or(false, |s| s == &structure.name);

                                            if ui
                                                .selectable_label(selected, &structure.name)
                                                .clicked()
                                            {
                                                select_structure = Some(structure.name.clone());
                                            }
                                        }
                                    });
                                }
                            }
                        });
                    }
                }

                // 상태 업데이트
                if let Some(category_name) = toggle_category {
                    let entry = self
                        .expanded_categories
                        .entry(category_name)
                        .or_insert(true);
                    *entry = !*entry;
                }

                if let Some((category_name, subcategory_name)) = toggle_subcategory {
                    let key = format!("{}-{}", category_name, subcategory_name);
                    let entry = self.expanded_subcategories.entry(key).or_insert(true);
                    *entry = !*entry;
                }

                if let Some(structure_name) = select_structure {
                    self.selected_structure = Some(structure_name.clone());
                    self.show_setting_panel = false;

                    // 선택된 구조체의 데이터 불러오기
                    if !self.erp_data.data.contains_key(&structure_name) {
                        self.load_structure_data(&structure_name);
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
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            if let Some(selected_structure_name) = &self.selected_structure {
                self.save_to_csv(selected_structure_name);
            }
        }
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
