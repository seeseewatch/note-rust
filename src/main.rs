mod state;
mod ui;

use state::AppState;

/// 程序的入口点。
///
/// eframe::run_native 接管事件循环的所有权——类似于 Zed 中 App::run() 的工作方式。
/// 我们需要提供三个要素：
///   1. 窗口标题
///   2. 原生窗口选项（大小、图标等）
///   3. 一个工厂闭包，返回应用状态（AppState）
///
/// 初始化过程中的任何错误都会返回并打印到 stderr。
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            // 初始标题为 "Untitled"，当用户打开或保存文件后会被更新
            .with_title("Untitled - NoteRust"),
        ..Default::default()
    };

    // run_native 会阻塞当前线程并进入 egui 的主事件循环。
    // 第三个参数是一个工厂闭包：egui 在启动时调用它来创建 AppState。
    // _creation_context 提供了对 wgpu 渲染状态和系统信息的访问，
    // v1 版本暂不需要用到。
    eframe::run_native(
        "NoteRust",
        options,
        Box::new(|_creation_context| Ok(Box::new(AppState::default()))),
    )
}
