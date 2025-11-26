use crate::slint_generatedAppWindow;
use aim_chart::Chart;
use aim_data::explorer::vci::OrderList;
pub use cache_storage::spawn_cache_storage_task;
pub use chart::*;
pub use dashboard::*;
pub use market_watch::*;
use slint_generatedAppWindow::{
    MarketWatchData as SlintMarketWatchData, StockData as SlintStockData,
};
use std::{fs::File, io::Write, path::PathBuf};

pub mod backend;
pub mod cache_storage;
pub mod chart;
pub mod dashboard;
pub mod market_watch;
pub mod task_manager;
pub mod world_index;

/// Simplified macro for non-stock-specific tasks (dashboard, market watch, etc.)
#[macro_export]
macro_rules! create_simple_task {
    (
        $task_fn:ident,
        $task_id:literal,
        $task_description:literal,
        $fetch_fn:ident,
        $ui_setter:ident,
        $data_type:ty,
        $ui_conversion:expr,
        $update_interval:literal
    ) => {
        pub async fn $task_fn(ui: &$crate::AppWindow) -> $crate::tasks::task_manager::TaskHandle {
            use slint::ComponentHandle;
            use $crate::tasks::task_manager::{register_task, TaskStatus};

            let ui_handle = ui.as_weak();
            let (tx, mut rx) = tokio::sync::mpsc::channel(10);
            let task_handle =
                register_task($task_id.to_string(), tx, $task_description.to_string()).await;

            tokio::spawn(async move {
                let mut task_status = TaskStatus::Running;

                loop {
                    // Check for task status updates
                    if let Ok(status) = rx.try_recv() {
                        if task_status != status {
                            log::info!(
                                "{} task status changed to: {:?}",
                                $task_description,
                                status
                            );
                            task_status = status;
                        }
                    }

                    // Skip processing if not running
                    if task_status != TaskStatus::Running {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }

                    // Fetch and update data
                    match $fetch_fn().await {
                        Ok(data) => {
                            let ui_handle_clone = ui_handle.clone();
                            let _ = ui_handle_clone.upgrade_in_event_loop(move |ui| {
                                let ui_data = $ui_conversion(data);
                                ui.$ui_setter(ui_data);
                                log::info!("Updated {} data", $task_description);
                            });
                        }
                        Err(e) => {
                            log::error!("Failed to fetch {} data: {}", $task_description, e);
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_millis($update_interval)).await;
                }
            });

            task_handle
        }
    };
}

#[repr(C)]
pub struct ChartMetaData {
    data: Vec<Chart>,
}

impl ChartMetaData {
    pub fn new(data: Vec<Chart>) -> Self {
        Self { data }
    }

    pub fn save(&self, mut file: File) {
        const VERSION: u32 = 1;

        let mut bytes = Vec::new();
        // Write version header
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        let count = self.data.len() as u32;
        bytes.extend_from_slice(&count.to_le_bytes());
        for (i, chart) in self.data.iter().enumerate() {
            let start_len = bytes.len();
            chart.write_to_bytes(&mut bytes);
            let written = bytes.len() - start_len;
            if written == 0 {
                log::error!("Chart #{} failed to serialize: {}", i, chart.stock_name);
            }
        }
        if let Err(e) = file.write_all(&bytes) {
            log::error!("Failed to write chart data to cache file: {e}");
        }
    }

    // Load charts from a file (manual deserialization, no external crate)
    pub fn load(path: &PathBuf) -> Self {
        let mut data = Vec::new();
        match std::fs::read(path) {
            Ok(bytes) => {
                let mut pos = 0;
                if bytes.len() < 8 {
                    log::error!("File too small to contain version and chart count: {path:?}");
                    return Self { data };
                }
                let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                pos += 4;
                if version != 1 {
                    log::error!("Unsupported chart cache version {version} in {path:?}");
                    return Self { data };
                }
                let count = u32::from_le_bytes([
                    bytes[pos],
                    bytes[pos + 1],
                    bytes[pos + 2],
                    bytes[pos + 3],
                ]) as usize;
                pos += 4;
                for i in 0..count {
                    let chart_bytes = &bytes[pos..];
                    match Chart::read_from_bytes(chart_bytes) {
                        Some((chart, used)) => {
                            if used == 0 {
                                log::error!(
                                    "Chart #{i} deserialized 0 bytes at pos {pos} in {path:?}"
                                );
                                break;
                            }
                            data.push(chart);
                            pos += used;
                        }
                        None => {
                            log::error!(
                                "Failed to deserialize chart #{i} at pos {pos} in {path:?}"
                            );
                            break;
                        }
                    }
                }
                log::info!("Loaded {} charts from {}", data.len(), path.display());
                for chart in &data {
                    log::info!("Chart loaded: {}", chart.stock_name);
                }
            }
            Err(e) => {
                log::error!("Failed to read chart file {path:?}: {e}");
            }
        }

        log::error!(
            "Chart data loaded from {}: {} charts",
            path.display(),
            data.len()
        );
        for chart in &data {
            log::info!("Chart loaded: {}", chart.stock_name);
        }
        Self { data }
    }

    // Get a simple hash of the chart data (no external crate)
    pub fn get_md5(&self) -> String {
        // Use a simple FNV-1a hash for demonstration
        let mut hash: u64 = 0xcbf29ce484222325;
        for chart in &self.data {
            let bytes = chart.to_bytes();
            for b in bytes {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        format!("{hash:016x}")
    }
}

pub enum DataUpdate {
    MarketWatchData(Vec<SlintMarketWatchData>),
    StockData(Vec<SlintStockData>),
    OrdList(OrderList),
    CustomList(Vec<String>),
}
