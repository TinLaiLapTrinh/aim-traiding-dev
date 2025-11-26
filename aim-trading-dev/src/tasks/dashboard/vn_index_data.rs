// File: src/tasks/dashboard/vn_index_data.rs
use crate::create_simple_task;
// Sửa lỗi 2: Import struct UI với tên chính xác
use crate::slint_generatedAppWindow::VnIndexData;
// Import struct API và hàm fetch
use aim_data::aim::{fetch_vn_index_data, VnIndexDataFetching};
use slint::{ModelRc, VecModel};


fn convert_vn_index_to_ui_data(api_data: Vec<VnIndexDataFetching>) -> ModelRc<VnIndexData> {
    // 1. Chuyển đổi từ kiểu API sang kiểu UI (Map)
    let ui_data: Vec<VnIndexData> = api_data
        .into_iter() // Sử dụng .into_iter() để chuyển ownership
        .map(|item| {
            // Mapping từng trường từ VnIndexDataFetching sang IndexData
            VnIndexData {
                // Ép kiểu (cast) u64 -> i32/i64 (int trong Slint)
                trading_date: item.trading_date as i32,
                // String -> string (sử dụng .into() hoặc .to_string().into())
                stock_code: item.stock_code.into(),

                // Sửa lỗi 2: Ép kiểu f64 -> f32 cho tất cả các trường float/f64
                pe: item.pe as f32,
                pb: item.pb as f32,
                // Sửa lỗi 1: Thêm trường ps và ép kiểu
                ps: item.ps as f32,
                close_price: item.close_price as f32,
            }
        })
        .collect();

    
        println!(">>> UI DATA after convert:\n{:#?}", ui_data);

    // 2. Chuyển đổi sang VecModel và ModelRc
    ModelRc::new(VecModel::from(ui_data))
}

// -------------------------------------------------------------------------
// MACRO TẠO TASK ĐÃ SỬA LỖI 1
// -------------------------------------------------------------------------
create_simple_task!(
    spawn_vn_index_task,
    "dashboard.vn_index",
    "VN Index Data",
    // $fetch_fn: Hàm này trả về Vec<VnIndexDataFetching>
    fetch_vn_index_data,
    // $ui_setter: Sửa lỗi 2: Dùng tên setter chính xác theo gợi ý compiler
    set_vn_ix_data,
    // $data_type: Sửa lỗi 1: Khai báo đúng kiểu dữ liệu mà fetch_vn_index_data trả về
    Vec<VnIndexDataFetching>,
    // $ui_conversion: Hàm chuyển đổi đã sửa
    convert_vn_index_to_ui_data,
    2000
);
