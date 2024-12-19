# Rust ERP System Documentation

## 개요
이 프로젝트는 Rust로 작성된  ERP 시스템입니다. egui 프레임워크를 사용하여 구축된 데스크톱 애플리케이션으로, 유연한 데이터 구조 관리와 직관적인 사용자 인터페이스를 제공합니다.

## 주요 기능

### 1. 데이터 구조 관리
- 계층적 구조 지원
  - 카테고리
  - 서브카테고리
  - 커스텀 구조체
- 유연한 필드 타입 시스템
  - Text (텍스트)
  - Number (숫자)
  - Date (날짜)
  - Boolean (참/거짓)

### 2. 데이터 처리
- 실시간 데이터 입력 및 편집
- JSON 기반 데이터 저장
- CSV 자동 백업
- Excel 파일 가져오기/내보내기

### 3. 사용자 인터페이스
- 직관적인 사이드바 네비게이션
- 구조체 편집기
- 데이터 입력 폼
- 카테고리 관리 시스템

## 시스템 구조

### 핵심 구조체
```rust
struct ERPApp {
    custom_structures: Vec<CustomCategory>,     // 커스텀 구조체 목록
    current_structure: CustomStructure,         // 현재 구조체
    erp_data: ERPData,                         // ERP 데이터
    // ... 기타 필드
}
```

### 데이터 모델
```rust
struct CustomCategory {
    name: String,
    subcategories: Vec<SubCategory>,
}

struct SubCategory {
    name: String,
    structures: Vec<CustomStructure>,
}

struct CustomStructure {
    name: String,
    fields: Vec<Field>,
}
```

## 데이터 저장

### 파일 형식
1. **구조체 정의**: `custom_structures.json`
   - 카테고리, 서브카테고리, 구조체 정의 저장
   - JSON 형식

2. **ERP 데이터**: `erp_data.json`
   - 실제 입력된 데이터 저장
   - JSON 형식

3. **백업 데이터**: `[structure_name].csv`
   - 구조체별 데이터 자동 백업
   - CSV 형식

## 기능 상세

### Excel 통합
```rust
// Excel 내보내기
fn export_to_excel(&self, structure: &CustomStructure) -> Result<(), Box<dyn Error>>

// Excel 가져오기
fn import_from_excel(&mut self, structure: &CustomStructure) -> Result<(), Box<dyn Error>>
```

### 데이터 관리
```rust
// 데이터 저장
fn save_erp_data(&self)

// 데이터 로드
fn load_erp_data(&mut self)

// CSV 백업
fn save_to_csv(&self, structure_name: &str)
```

## 사용된 주요 크레이트
- `eframe`: GUI 프레임워크
- `serde`: 직렬화/역직렬화
- `calamine`: Excel 파일 읽기
- `xlsxwriter`: Excel 파일 쓰기
- `csv`: CSV 파일 처리

## 향후 개선 사항
1. 데이터 검증 시스템 추가
2. 사용자 권한 관리
3. 네트워크 동기화 기능
4. 데이터 백업 및 복원 시스템 강화
5. 검색 및 필터링 기능 개선

## 개발 환경 설정

### 필수 요구사항
- 필요한 크레이트:
```toml
[dependencies]
eframe = "0.29.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
xlsxwriter = "0.6.1"
calamine = "0.21.1"
csv = "1.2"
rfd = "0.11"

  ```

### 빌드 및 실행
```bash
# 프로젝트 빌드
cargo build --release

# 실행
cargo run --release
```

