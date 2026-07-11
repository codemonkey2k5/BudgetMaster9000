import { invoke } from "@tauri-apps/api/core";

export interface AppStatus {
  locked: boolean;
  lockEnabled: boolean;
  dbPath: string;
  portable: boolean;
  hasData: boolean;
  theme: string;
  needsCategoryReview: boolean;
}

export interface IncomeSettings {
  annualSalary: number;
  taxBracket: number;
  grossMonthly: number;
  netMonthly: number;
  biweeklyPay: number;
}

export interface Category {
  id: number;
  name: string;
  color: string;
  sortOrder: number;
  isFixed: boolean;
}

export interface CategoryInput {
  id?: number | null;
  name: string;
  isFixed: boolean;
}

export interface BudgetLine {
  id: number;
  name: string;
  categoryId: number;
  categoryName: string;
  amount: number;
  frequency: string;
  monthlyAmount: number;
  isFixed: boolean;
  notes: string;
  active: boolean;
}

export interface BudgetLineInput {
  id?: number | null;
  name: string;
  categoryId: number;
  amount: number;
  frequency: string;
  isFixed?: boolean | null;
  notes?: string | null;
}

export interface MonthInfo {
  id: number;
  year: number;
  month: number;
  status: string;
  netIncome: number;
  notes: string;
  mood: string | null;
  grade: string | null;
  closedAt: string | null;
}

export interface MonthLine {
  budgetLineId: number;
  name: string;
  categoryId: number;
  categoryName: string;
  categoryColor: string;
  budgetAmount: number;
  actualAmount: number | null;
  isFixed: boolean;
  notes: string;
  variance: number | null;
  pctUsed: number | null;
  status: string;
}

export interface StatusCounts {
  under: number;
  onPlan: number;
  over: number;
  unset: number;
}

export interface MonthDashboard {
  month: MonthInfo;
  lines: MonthLine[];
  budgetedTotal: number;
  actualTotal: number;
  varianceTotal: number;
  savingsRate: number | null;
  counts: StatusCounts;
}

export interface ActualInput {
  budgetLineId: number;
  actualAmount: number;
  notes?: string | null;
}

export interface CheckInResult {
  grade: string;
  score: number;
  savingsRate: number;
  wins: string[];
  attention: string[];
  trends: string[];
  suggestion: string | null;
  counts: StatusCounts;
}

export const api = {
  getStatus: () => invoke<AppStatus>("get_status"),
  unlock: (password: string) => invoke<boolean>("unlock", { password }),
  lockApp: () => invoke<void>("lock_app"),
  setAppLock: (password: string, enable: boolean) =>
    invoke<void>("set_app_lock", { password, enable }),
  setTheme: (theme: string) => invoke<void>("set_theme", { theme }),
  getIncome: () => invoke<IncomeSettings>("get_income"),
  saveIncome: (income: IncomeSettings) => invoke<void>("save_income", { income }),
  listCategories: () => invoke<Category[]>("list_categories"),
  addCategory: (name: string) => invoke<Category>("add_category", { name }),
  upsertCategory: (category: CategoryInput) =>
    invoke<Category>("upsert_category", { category }),
  deleteCategory: (id: number) => invoke<void>("delete_category", { id }),
  dismissCategoryReview: () => invoke<void>("dismiss_category_review"),
  listBudgetLines: () => invoke<BudgetLine[]>("list_budget_lines"),
  upsertBudgetLine: (line: BudgetLineInput) =>
    invoke<number>("upsert_budget_line", { line }),
  deleteBudgetLine: (id: number) => invoke<void>("delete_budget_line", { id }),
  getDashboard: (year: number, month: number) =>
    invoke<MonthDashboard>("get_dashboard", { year, month }),
  saveActuals: (year: number, month: number, actuals: ActualInput[]) =>
    invoke<void>("save_actuals", { year, month, actuals }),
  updateMonthMeta: (
    year: number,
    month: number,
    netIncome?: number | null,
    notes?: string | null,
    mood?: string | null
  ) =>
    invoke<void>("update_month_meta", {
      year,
      month,
      netIncome: netIncome ?? null,
      notes: notes ?? null,
      mood: mood ?? null,
    }),
  completeCheckIn: (
    year: number,
    month: number,
    actuals: ActualInput[],
    mood?: string | null,
    notes?: string | null
  ) =>
    invoke<CheckInResult>("complete_check_in", {
      year,
      month,
      actuals,
      mood: mood ?? null,
      notes: notes ?? null,
    }),
  listHistory: () => invoke<MonthInfo[]>("list_history"),
  reopenMonth: (year: number, month: number) =>
    invoke<void>("reopen_month", { year, month }),
  resyncMonth: (year: number, month: number) =>
    invoke<void>("resync_month", { year, month }),
  importLegacyJson: (json: string) =>
    invoke<string>("import_legacy_json", { json }),
  exportJson: () => invoke<string>("export_json"),
  loadDemoData: () => invoke<void>("load_demo_data"),
  currentMonth: () => invoke<[number, number]>("current_month"),
  reportUiSelftest: (ok: boolean, report: string) =>
    invoke<string>("report_ui_selftest", { ok, report }),
  clearAllData: (confirmPhrase: string) =>
    invoke<void>("clear_all_data", { confirmPhrase }),
};

export function money(n: number): string {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
  }).format(n || 0);
}

/** Format for amount inputs: 119.00 (no $ so number parsing works). */
export function amountInput(n: number): string {
  return (Number(n) || 0).toFixed(2);
}

export function monthName(m: number): string {
  return (
    [
      "January",
      "February",
      "March",
      "April",
      "May",
      "June",
      "July",
      "August",
      "September",
      "October",
      "November",
      "December",
    ][m - 1] || String(m)
  );
}

export function statusLabel(s: string): string {
  switch (s) {
    case "under":
      return "Under budget";
    case "on_plan":
      return "On plan";
    case "over":
      return "Over budget";
    default:
      return "Not entered yet";
  }
}

export function gradeExplain(g: string | null | undefined): string {
  if (!g) return "No grade yet. Finish Check-In to score this month.";
  const map: Record<string, string> = {
    A: "Excellent: most spending on track with a strong savings rate.",
    "B+": "Very good: mostly on plan with only small slips.",
    B: "Good: solid overall, a few areas to watch.",
    "B-": "Okay: more overspends or a thinner savings rate.",
    "C+": "Mixed: several categories need attention.",
    C: "Needs work: budget was missed in important areas.",
    D: "Difficult month: review the Plan and next Check-In carefully.",
    F: "Far off plan: use the scorecard tips to reset next month.",
  };
  return map[g] || `Grade ${g} from your Check-In scorecard.`;
}
