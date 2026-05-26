use std::path::PathBuf;

/// 应用的核心状态。
///
/// 设计思路（参考 Zed 的 Entity<T> 模式，但极度简化）：
/// - text:   文本缓冲区，所有编辑操作都通过 egui 的 TextEdit 直接修改它
/// - file_path: 当前文件的路径。None 表示"新建、尚未保存的文件"（与记事本行为一致）
/// - dirty:  自上次保存以来文本是否发生了变化。用于在标题栏显示 "*" 标记
#[derive(Default)]
pub struct AppState {
    pub text: String,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
}

impl AppState {
    /// 重置所有状态来模拟"新建文件"操作。
    ///
    /// 需要显式重置 dirty 标志（而非通过比较文本判断），
    /// 因为用户可能在执行新建前就已经修改了内容。
    pub fn new_file(&mut self) {
        self.text.clear();
        self.file_path = None;
        self.dirty = false;
    }

    /// 打开文件对话框并读取内容。
    ///
    /// rfd::AsyncFileDialog 是非阻塞的——它返回一个 Future。
    /// 在 egui 中无法直接 .await，因此我们使用异步文件对话框 API：
    /// 轮询对话框直到用户做出选择，从选中的文件中读取内容。
    pub fn open_file(&mut self) {
        // rfd 的异步对话框：pick_file() 不阻塞主线程，
        // 返回一个在用户选择文件后 resolve 的 Future。
        // 这里使用同步版本的 FileDialog 来简化实现。
        // 在 v1 版本中，show() 会短暂阻塞但用户感知不到延迟。
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            // std::fs::read_to_string 将整个文件读入内存。
            // 对于非常大的文件（> 几百 MB），这会导致卡顿——
            // 对于 v1 记事本级别的编辑器来说可以接受。
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.text = content;
                    self.file_path = Some(path);
                    self.dirty = false;
                }
                Err(err) => {
                    // 打开失败时保留旧状态，只打印错误。
                    // egui 没有内置的通知系统，所以这里用 eprintln。
                    eprintln!("Failed to open file: {err}");
                }
            }
        }
    }

    /// 保存当前文本到已关联的文件路径。
    /// 如果没有路径（新建文件），则回退到 save_as。
    pub fn save_file(&mut self) {
        if let Some(ref path) = self.file_path {
            // write() 直接覆盖文件内容。
            // 这与 Zed 的 Buffer::save() 类似——原子写入不是 v1 的需求。
            match std::fs::write(path, &self.text) {
                Ok(()) => self.dirty = false,
                Err(err) => eprintln!("Failed to save file: {err}"),
            }
        } else {
            // 没有关联路径：让用户在保存对话框中选择位置。
            self.save_file_as();
        }
    }

    /// 始终弹出"另存为"对话框，并让用户选择一个路径。
    pub fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            match std::fs::write(&path, &self.text) {
                Ok(()) => {
                    // 保存成功后，将新路径关联到当前文件。
                    // 后续的"保存"操作将直接写入此路径，不再弹出对话框。
                    self.file_path = Some(path);
                    self.dirty = false;
                }
                Err(err) => eprintln!("Failed to save file: {err}"),
            }
        }
    }

    /// 返回当前文件的文件名（不含路径），用于窗口标题。
    /// "新建文件"时返回 "Untitled"。
    pub fn file_name(&self) -> &str {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
    }
}
