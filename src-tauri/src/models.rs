use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub locked: bool,
    pub lock_enabled: bool,
    pub db_path: String,
    pub portable: bool,
    pub has_data: bool,
    pub theme: String,
    /// True after importing old data until user dismisses the category-review notice.
    pub needs_category_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeSettings {
    pub annual_salary: f64,
    pub tax_bracket: f64,
    pub gross_monthly: f64,
    pub net_monthly: f64,
    pub biweekly_pay: f64,
}

impl Default for IncomeSettings {
    fn default() -> Self {
        Self {
            annual_salary: 0.0,
            tax_bracket: 0.0,
            gross_monthly: 0.0,
            net_monthly: 0.0,
            biweekly_pay: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    /// When true, every budget line in this category is treated as a fixed bill.
    pub is_fixed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInput {
    pub id: Option<i64>,
    pub name: String,
    pub is_fixed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetLine {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub category_name: String,
    pub amount: f64,
    pub frequency: String,
    pub monthly_amount: f64,
    pub is_fixed: bool,
    pub notes: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetLineInput {
    pub id: Option<i64>,
    pub name: String,
    pub category_id: i64,
    pub amount: f64,
    pub frequency: String,
    /// Ignored on write — fixed/flexible comes from the category.
    #[serde(default)]
    pub is_fixed: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthInfo {
    pub id: i64,
    pub year: i32,
    pub month: i32,
    pub status: String,
    pub net_income: f64,
    pub notes: String,
    pub mood: Option<String>,
    pub grade: Option<String>,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthLine {
    pub budget_line_id: i64,
    pub name: String,
    pub category_id: i64,
    pub category_name: String,
    pub category_color: String,
    pub budget_amount: f64,
    pub actual_amount: Option<f64>,
    pub is_fixed: bool,
    pub notes: String,
    pub variance: Option<f64>,
    pub pct_used: Option<f64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthDashboard {
    pub month: MonthInfo,
    pub lines: Vec<MonthLine>,
    pub budgeted_total: f64,
    pub actual_total: f64,
    pub variance_total: f64,
    pub savings_rate: Option<f64>,
    pub counts: StatusCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusCounts {
    pub under: i32,
    pub on_plan: i32,
    pub over: i32,
    pub unset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualInput {
    pub budget_line_id: i64,
    pub actual_amount: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckInResult {
    pub grade: String,
    pub score: f64,
    pub savings_rate: f64,
    pub wins: Vec<String>,
    pub attention: Vec<String>,
    pub trends: Vec<String>,
    pub suggestion: Option<String>,
    pub counts: StatusCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyImport {
    pub last_edit: Option<String>,
    pub income: Option<LegacyIncome>,
    pub categories: Option<Vec<String>>,
    pub expenses: Option<Vec<LegacyExpense>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyIncome {
    pub annual_salary: Option<f64>,
    pub tax_bracket: Option<f64>,
    pub gross_monthly: Option<f64>,
    pub net_monthly: Option<f64>,
    pub biweekly_pay: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyExpense {
    pub id: Option<String>,
    pub name: String,
    pub category: String,
    pub amount: f64,
    pub frequency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBundle {
    pub version: u32,
    pub exported_at: String,
    pub income: IncomeSettings,
    pub categories: Vec<Category>,
    pub budget_lines: Vec<BudgetLine>,
    pub months: Vec<MonthExport>,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthExport {
    pub year: i32,
    pub month: i32,
    pub status: String,
    pub net_income: f64,
    pub notes: String,
    pub mood: Option<String>,
    pub grade: Option<String>,
    pub actuals: Vec<ActualExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualExport {
    pub line_name: String,
    pub category_name: String,
    pub budget_amount: f64,
    pub actual_amount: f64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_url: String,
    pub zip_asset_name: Option<String>,
    pub checked: bool,
    pub error: Option<String>,
}

/// One dated spend (or adjustment) event on a month budget line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineTransaction {
    pub id: i64,
    pub year: i32,
    pub month: i32,
    pub budget_line_id: i64,
    pub line_name: String,
    pub category_id: i64,
    pub category_name: String,
    pub category_color: String,
    pub is_fixed: bool,
    pub amount: f64,
    pub occurred_on: String,
    pub notes: String,
    pub source: String,
    pub created_at: String,
}
