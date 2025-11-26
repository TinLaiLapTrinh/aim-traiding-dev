// Include all Slint UI modules
slint::include_modules!();

// Import required modules
mod task_manager;
mod tasks;
use crate::{
    slint_generatedAppWindow::StockData as SlintStockData,
    tasks::{
        sort_market_watch, sort_stocks, spawn_cache_storage_task, ChartMetaData, ALL_STOCK_LIST,
    },
};
use aim_chart::Chart;
use aim_data::{get_company_info, get_market_watch, get_quote};
use dirs_next::cache_dir;
use slint::{Model, SharedString, VecModel};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

// Import task functions
use tasks::{
    convert_to_stock_data, spawn_abnormal_trade_task, spawn_balance_sheet_task,
    spawn_chart_update_task, spawn_company_profile_task, spawn_data_update_task,
    spawn_heat_map_task, spawn_icb_index_task, spawn_mini_chart_hnx30_task,
    spawn_mini_chart_hnxindex_task, spawn_mini_chart_vn30_task, spawn_mini_chart_vnindex_task,
    spawn_overall_index_task, spawn_sjc_price_task, spawn_stock_influence_task,
    spawn_stock_update_task, spawn_trading_volume_task, spawn_ui_chart_task,
    spawn_finance_report_task, spawn_finance_pdf_selected_task, render_pdf_to_png_paths
    ,spawn_vn_index_task
};
use aim_data::aim::fetch_finance_report_pdf;
// use crate::tasks::render_pdf_to_png_paths;


static MY_STOCK_LIST: [&str; 1] = ["AAA"];


/// Main entry point of the application
/// This function sets up the trading application with real-time stock data visualization
#[tokio::main]
async fn main() {

    // #[cfg(all(target_os = "windows", not(debug_assertions)))]
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
    
    
    // Initialize logging with Info level
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Error)
        .init();

    let base_cache = cache_dir().expect("Could not find cache directory");
    let app_cache_dir = base_cache.join("Aim");
    std::fs::create_dir_all(&app_cache_dir).unwrap();
    let cache_file: PathBuf = app_cache_dir.join("cache.bin");
    let user_list: PathBuf = app_cache_dir.join("user_list.json");

    // Fetch initial chart data for default stock (AAA)
    let chart_data = get_quote(&["AAA"], "ONE_DAY", None, None).await.unwrap();
    let company_info = get_company_info("AAA").await.unwrap();
    let company_info = aim_chart::CompanyInfo {
        roe: company_info.data.company_financial_ratio.ratio[0]
            .roe
            .unwrap_or(0.0),
        roa: company_info.data.company_financial_ratio.ratio[0]
            .roa
            .unwrap_or(0.0),
        pe: company_info.data.company_financial_ratio.ratio[0]
            .pe
            .unwrap_or(0.0),
        pb: company_info.data.company_financial_ratio.ratio[0]
            .pb
            .unwrap_or(0.0),
        eps: company_info.data.company_financial_ratio.ratio[0]
            .eps
            .unwrap_or(0.0),
    };

    // Create a thread-safe chart container with initial chart
    let chart_metadata = if std::fs::metadata(&cache_file).is_ok() {
        ChartMetaData::load(&cache_file)
    } else {
        ChartMetaData::new(vec![Chart::new_default(
            "AAA".to_string(),
            chart_data.0[0].clone(),
            company_info,
        )])
    };
    let chart = Arc::new(Mutex::new(chart_metadata));

    let init_data = match get_market_watch(&["AAA"]).await {
        Ok(stock_list_data) => {
            let stock_data: Vec<SlintStockData> = stock_list_data
                .0
                .iter()
                .map(convert_to_stock_data)
                .collect();
            stock_data[0].clone()
        }
        Err(e) => {
            log::error!("Failed to fetch stock data: {e}.");
            SlintStockData::default()
        }
    };

    // Initialize the main UI window
    let ui = slint_generatedAppWindow::AppWindow::new().unwrap();

    // Initialize page-aware task manager
    task_manager::initialize_page_manager(&ui).await;
    log::info!("Page-aware task manager initialized");

    let default_user_list = Arc::new(Mutex::new(
        MY_STOCK_LIST
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    ));
    // Initialize the symbol list with initial values
    let symbol_list = if std::fs::metadata(&user_list).is_ok() {
        if let Ok(custom_list_data) = std::fs::read_to_string(&user_list) {
            if let Ok(custom_list) = serde_json::from_str::<Vec<String>>(&custom_list_data) {
                Arc::new(Mutex::new(custom_list))
            } else {
                log::error!("Failed to parse custom_list.json");
                default_user_list.clone()
            }
        } else {
            default_user_list.clone()
        }
    } else {
        default_user_list
    };

    ui.set_current_stock(init_data);

    // Set up callback for adding symbols
    let symbol_list_clone = Arc::clone(&symbol_list);
    let ui_handle: slint::Weak<AppWindow> = ui.as_weak();
    ui.on_add_stock(move |group_name: SharedString, symbol: SharedString| {
        if group_name == "MY LIST" {
            let symbol = symbol.to_uppercase();
            let symbol_list_clone = symbol_list_clone.clone();
            ui_handle.unwrap().set_is_list_in_update(true);
            let ui_handle = ui_handle.clone();
            tokio::spawn(async move {
                if !ALL_STOCK_LIST.contains(&symbol.as_str()) {
                    log::error!("Failed to add stock: {symbol} - not found in ALL_STOCK_LIST");
                } else {
                    let mut list = symbol_list_clone.lock().await;
                    if !list.contains(&symbol.to_string()) {
                        list.push(symbol.to_string());
                    } else {
                        log::warn!("Stock {symbol} already exists in the list {list:?}");
                    }
                }
                ui_handle.upgrade_in_event_loop(move |ui| {
                    ui.set_is_list_in_update(false);
                })
            });
        }
    });

    // Set up callback for removing symbols
    let symbol_list_clone = Arc::clone(&symbol_list);
    let ui_handle: slint::Weak<AppWindow> = ui.as_weak();
    ui.on_remove_stock(move |group_name: SharedString, symbol: SharedString| {
        if group_name == "MY LIST" {
            let symbol = symbol.to_uppercase();
            let symbol_list_clone = symbol_list_clone.clone();
            ui_handle.unwrap().set_is_list_in_update(true);
            let ui_handle = ui_handle.clone();
            tokio::spawn(async move {
                let mut list = symbol_list_clone.lock().await;
                list.retain(|s| s != &symbol.to_string());
                ui_handle.upgrade_in_event_loop(move |ui| {
                    ui.set_is_list_in_update(false);
                })
            });
        }
    });

    // Set up callback for toggling group expansion
    ui.on_toggle_group(move |group_idx: i32| {
        log::info!("Toggling group {group_idx}");
        // For now, just log the group toggle - you can implement actual state management here
        // In a real implementation, you might want to store group expansion state
    });

    // Set up callback for switching watchlists
    let ui_handle_clone = ui.as_weak();
    ui.on_switch_list(move |list_name: slint::SharedString| {
        log::info!("Switching to watchlist: {list_name}");
        let ui_handle = ui_handle_clone.clone();

        // Trigger data update with new category filter
        tokio::spawn(async move {
            // You can implement specific logic here to filter stocks by category
            // For now, we'll just log the switch
            let _ = ui_handle.upgrade_in_event_loop(move |ui| {
                ui.set_is_list_in_update(true);
                // In a real implementation, you would filter the data here
                ui.set_is_list_in_update(false);
            });
        });
    });

    let ui_handle = ui.as_weak();
    ui.on_sort_stocks(move |sort_type| {
        let ui_handle_clone = ui_handle.clone();
        tokio::spawn(async move {
            let _ = ui_handle_clone.upgrade_in_event_loop(move |ui| {
                let stock_groups = ui.get_stock_groups();
                let stock_groups_vec: Vec<_> = (0..stock_groups.row_count())
                    .map(|i| stock_groups.row_data(i).unwrap())
                    .collect();
                let updated_groups = sort_stocks(&stock_groups_vec, sort_type);

                // Update the UI with sorted groups
                ui.set_stock_groups(slint::ModelRc::new(VecModel::from(updated_groups)));
            });
        });
    });

    let ui_handle_market_watch = ui.as_weak();
    ui.on_sort_market_watch(move |sort_column| {
        let ui_handle_clone = ui_handle_market_watch.clone();
        tokio::spawn(async move {
            let _ = ui_handle_clone.upgrade_in_event_loop(move |ui| {
                let market_data = ui.get_market_watch_data();
                let market_data_vec: Vec<_> = (0..market_data.row_count())
                    .map(|i| market_data.row_data(i).unwrap())
                    .collect();

                let sort_ascending = ui.get_market_watch_sort_ascending();
                let show_percentage = ui.get_market_watch_show_percentage();
                let sorted_data = sort_market_watch(
                    &market_data_vec,
                    sort_column,
                    sort_ascending,
                    show_percentage,
                );

                // Update the UI with sorted data
                ui.set_market_watch_data(slint::ModelRc::new(VecModel::from(sorted_data)));
            });
        });
    });

    let ui_handle_selected_pdf = ui.as_weak();
    ui.on_report_selected(move |report_id| {
        let ui_handle_clone = ui_handle_selected_pdf.clone();
        tokio::spawn(async move {
            // Log
            log::info!("[LOI]🟢 User selected report_id = {}", report_id);

            // 🟡 Báo cho UI biết đang tải
            let _ = ui_handle_clone.upgrade_in_event_loop(|ui| {
                ui.set_is_loading(true);
            });

            // Gọi API và render ngay ở đây (bỏ qua task nền)
            match fetch_finance_report_pdf(&report_id).await {
                Ok(pdf) => {
                    log::info!("[LOI]📄 Đã tải PDF báo cáo cho report_id={} -> {}", report_id, pdf.file_path);

                    match render_pdf_to_png_paths(&pdf.file_path).await {
                        Ok(png_paths) => {
                            log::info!("[LOI]🖼️ Render PDF -> PNG thành công: {} trang", png_paths.len());
                            for (i, path) in png_paths.iter().enumerate() {
                                log::debug!("[LOI] ↳ Trang {} -> {}", i + 1, path);
                            }

                            // Đưa kết quả lên UI thread
                            let _ = ui_handle_clone.upgrade_in_event_loop(move |ui| {
                                let mut slint_images: Vec<slint::Image> = Vec::new();

                                for p in png_paths.iter() {
                                    match slint::Image::load_from_path(std::path::Path::new(p)) {
                                        Ok(img) => {
                                            slint_images.push(img);
                                            log::debug!("[LOI]✅ Load ảnh thành công: {}", p);
                                        }
                                        Err(_) => {
                                            log::warn!("[LOI]⚠️ Không thể load ảnh {}", p);
                                        }
                                    }
                                }

                                let model: slint::ModelRc<slint::Image> =
                                    slint::ModelRc::new(slint::VecModel::from(slint_images));

                                ui.set_pdf_pages(model);
                                ui.set_selected_report_id(report_id.clone());
                                ui.set_is_loading(false); // 🟢 Tắt loading

                                log::info!("[LOI]✅ UI đã cập nhật báo cáo PDF cho report_id={}", report_id);
                            });
                        }
                        Err(e) => {
                            log::error!("[LOI]❌ Lỗi khi render PDF -> PNG: {:?}", e);
                            let _ = ui_handle_clone.upgrade_in_event_loop(|ui| ui.set_is_loading(false));
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Lỗi khi tải PDF từ API cho report_id={}: {:?}", report_id, e);
                    let _ = ui_handle_clone.upgrade_in_event_loop(|ui| ui.set_is_loading(false));
                }
            }

        });
    });


    // Spawn all the tasks
    let _ui_chart_handle = spawn_ui_chart_task(Arc::clone(&chart), &ui).await;
    // If you only want to read the chart data, you can pass a reference to the Arc<Mutex<ChartMetaData>>
    // Spawn cache storage task with task manager
    let _cache_handle =
        spawn_cache_storage_task(Arc::clone(&chart), Arc::clone(&symbol_list)).await;
    let _stock_update_handles = spawn_stock_update_task(Arc::clone(&chart), &ui).await;
    let _chart_update_handle = spawn_chart_update_task(Arc::clone(&chart)).await;
    let _data_update_handle = spawn_data_update_task(&ui, Arc::clone(&symbol_list)).await;
    let _balance_sheet_handles = spawn_balance_sheet_task(&ui).await;
    let _company_profile_handles = spawn_company_profile_task(&ui).await;
    let _mini_vnindex_handle = spawn_mini_chart_vnindex_task(&ui).await;
    let _mini_vn30_handle = spawn_mini_chart_vn30_task(&ui).await;
    let _mini_hnx30_handle = spawn_mini_chart_hnx30_task(&ui).await;
    let _mini_hnxindex_handle = spawn_mini_chart_hnxindex_task(&ui).await;
    // spawn_world_index_task(&ui);
    let _stock_influence_handle = spawn_stock_influence_task(&ui).await;
    let _vn_index_handle = spawn_vn_index_task(&ui).await;
    let _overall_index_handle = spawn_overall_index_task(&ui).await; // update OverallIndex UI component
    let _heat_map_handle = spawn_heat_map_task(&ui).await; // update HeatMap UI component
    let _icb_index_handle = spawn_icb_index_task(&ui).await; // update ICBIndex UI component
    let _abnormal_trade_handle = spawn_abnormal_trade_task(&ui).await; // update AbnormalTrade UI component
    let _trading_volume_handle = spawn_trading_volume_task(&ui).await; // update TradingVolume UI component
    let _sjc_price_handle = spawn_sjc_price_task(&ui).await; // update SJC price data for goods UI component
    // Gọi task xử lý Finance Report
    let _finance_report_task = spawn_finance_report_task(&ui).await; // update finance report data for goods UI component
    let _finance_pdf_selected_task = spawn_finance_pdf_selected_task(&ui).await; // update finance report data for goods UI component

    // Start active page monitoring after all tasks are spawned
    task_manager::start_page_monitoring(&ui).await;
    log::info!("Active page monitoring started - tasks will automatically pause/resume based on UI navigation");

    // Set up window close handler
    ui.window().on_close_requested(|| {
        log::info!("Closing the application...");
        std::process::exit(0);
    });

    // Run the UI main loop
    ui.run().unwrap();
}

/// Converts user-friendly interval strings to API interval constants
///
/// # Arguments
/// * `interval` - The interval string (e.g., "1m", "5m", "1H", "1D")
///
/// # Returns
/// The corresponding API interval constant
fn interval_to_constant(interval: &str) -> &'static str {
    match interval {
        "1m" | "5m" | "15m" | "30m" => "ONE_MINUTE",
        "1H" | "2H" | "4H" => "ONE_HOUR",
        "1D" | "2D" | "3D" => "ONE_DAY",
        "1W" | "2W" | "1M" => "ONE_WEEK",
        _ => "ONE_DAY",
    }
}
