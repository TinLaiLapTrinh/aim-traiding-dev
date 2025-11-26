use crate::explorer::aim::{
    AbnormalTrade, ExchangeIndex, FinanceSheetData, FinancialData, IcbIndex, InsiderTransaction,
    InstitutionData, Officer, PropTradingData, SharedHolder, SjcPriceData, StockByGics, Subsidiary,
    TopStockInfluencer, fetch_api_data, fetch_api_finance_report_pdf,
    StockReport, StrategyReport, PdfReport, ApiReport,VnIndexDataFetching
};


pub async fn fetch_balance_sheet_data(
    symbol: &str,
    period: &str,
) -> Result<Vec<FinanceSheetData>, reqwest::Error> {
    let endpoint = format!("balance-sheet/{symbol}/{period}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_cash_flow_gt_sheet_data(
    symbol: &str,
    period: &str,
) -> Result<Vec<FinanceSheetData>, reqwest::Error> {
    let endpoint = format!("cash-flow-indirect/{symbol}/{period}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_cash_flow_tt_sheet_data(
    symbol: &str,
    period: &str,
) -> Result<Vec<FinanceSheetData>, reqwest::Error> {
    let endpoint = format!("cash-flow-direct/{symbol}/{period}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_income_statement_sheet_data(
    symbol: &str,
    period: &str,
) -> Result<Vec<FinanceSheetData>, reqwest::Error> {
    let endpoint = format!("income-statement/{symbol}/{period}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_financial_data(symbol: &str) -> Result<Vec<FinancialData>, reqwest::Error> {
    let endpoint = format!("financial-data/{symbol}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_sharedholder_data(symbol: &str) -> Result<Vec<SharedHolder>, reqwest::Error> {
    let endpoint = format!("shareholder/{symbol}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_institution_data(symbol: &str) -> Result<InstitutionData, reqwest::Error> {
    let endpoint = format!("institution-profile/{symbol}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_subsidiaries_data(symbol: &str) -> Result<Vec<Subsidiary>, reqwest::Error> {
    let endpoins = format!("subsidiaries/{symbol}");
    let mut data: Vec<Subsidiary> = fetch_api_data(&endpoins).await?;

    // Remove duplicates based on institution_id, keeping the first occurrence
    let mut seen_ids = std::collections::HashSet::new();
    data.retain(|subsidiary| seen_ids.insert(subsidiary.institution_id));

    Ok(data)
}

pub async fn fetch_vn_index_data()->Result<Vec<VnIndexDataFetching>,reqwest::Error>{
    // let endpoint = format!("chart-vnindex-pe-pb");
    fetch_api_data("chart-vnindex-pe-pb").await
}

pub async fn fetch_officers_data(symbol: &str) -> Result<Vec<Officer>, reqwest::Error> {
    let endpoint = format!("officer/{symbol}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_insider_transactions_data(
    symbol: &str,
) -> Result<Vec<InsiderTransaction>, reqwest::Error> {
    let endpoint = format!("insider-transactions/{symbol}");
    fetch_api_data(&endpoint).await
}

pub async fn fetch_top_stock_influencer_data() -> Result<Vec<TopStockInfluencer>, reqwest::Error> {
    fetch_api_data("top-stock-influence").await
}

pub async fn fetch_exchange_index_data() -> Result<Vec<ExchangeIndex>, reqwest::Error> {
    fetch_api_data("exchange-index").await
}

pub async fn fetch_stock_by_gics_data() -> Result<Vec<StockByGics>, reqwest::Error> {
    fetch_api_data("stock-by-gics").await
}

pub async fn fetch_icb_index_data() -> Result<Vec<IcbIndex>, reqwest::Error> {
    fetch_api_data("icb-index").await
}

pub async fn fetch_abnormal_trade_data() -> Result<Vec<AbnormalTrade>, reqwest::Error> {
    fetch_api_data("abnormal-trades").await
}

pub async fn fetch_kqgd_td_chart_data() -> Result<Vec<PropTradingData>, reqwest::Error> {
    fetch_api_data("KQGD-TD-chart").await
}

pub async fn fetch_kqgd_nn_chart_data() -> Result<Vec<PropTradingData>, reqwest::Error> {
    fetch_api_data("KQGD-NN-chart").await
}

pub async fn fetch_sjc_price_data() -> Result<Vec<SjcPriceData>, reqwest::Error> {
    fetch_api_data("sjc-price").await
}

pub async fn fetch_finance_report_list() -> Result<Vec<ApiReport>, reqwest::Error> {
    let endpoint = "reports";
    fetch_api_data(&endpoint).await
}

/// 🔹 Lấy danh sách chiến lược đầu tư
pub async fn fetch_strategy_report_list() -> Result<Vec<StrategyReport>, reqwest::Error> {
    let endpoint = "reports?source=9999";
    fetch_api_data(&endpoint).await
}

/// 🔹 Lấy thông tin PDF của một báo cáo cụ thể
pub async fn fetch_finance_report_pdf(symbol: &str) -> Result<PdfReport, reqwest::Error> {
    // let endpoint = format!("report-file/{symbol}");
    fetch_api_finance_report_pdf(symbol).await
}



