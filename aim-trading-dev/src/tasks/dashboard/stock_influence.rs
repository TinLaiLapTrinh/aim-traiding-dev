use crate::create_simple_task;
use crate::slint_generatedAppWindow::InfluenceData;
use aim_data::aim::{fetch_top_stock_influencer_data, TopStockInfluencer};
use slint::{ModelRc, VecModel};

// Convert API TopStockInfluencer data to UI InfluenceData format
fn convert_stock_influence_to_ui_data(api_data: Vec<TopStockInfluencer>) -> ModelRc<InfluenceData> {
    // In thử dữ liệu API gốc
    // println!(">>> RAW API DATA:\n{:#?}", api_data);

    let mut ui_data: Vec<InfluenceData> = api_data
        .iter()
        .filter(|item| item.cat_id == 1)
        .map(|item| InfluenceData {
            symbol: item.stock_code.clone().into(),
            value: item.influence_index as f32,
        })
        .collect();

    // In dữ liệu sau khi convert
    // println!(">>> UI DATA BEFORE SORT:\n{:#?}", ui_data);

    ui_data.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // println!(">>> UI DATA AFTER SORT:\n{:#?}", ui_data);

    let model = VecModel::from(ui_data);
    ModelRc::new(model)
}


// Use the generic simple task pattern from tasks/mod.rs
create_simple_task!(
    spawn_stock_influence_task,
    "dashboard.stock_influence",
    "Stock Influence Data",
    fetch_top_stock_influencer_data,
    set_stock_influence_data,
    Vec<TopStockInfluencer>,
    convert_stock_influence_to_ui_data,
    2000 // Update every 2 seconds for stock influence data
);
