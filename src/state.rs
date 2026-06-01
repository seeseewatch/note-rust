use std::path::PathBuf;

/// 应用的核心状态。
///
/// 字段说明：
/// - text:         文本缓冲区，所有编辑操作都通过 egui 的 TextEdit 直接修改它
/// - file_path:    当前文件的路径，None 表示新建、尚未保存的文件
/// - dirty:        自上次保存以来文本是否发生了变化（标题栏 "*" 标记）
/// - show_line_numbers: 是否在左侧显示行号
/// - word_wrap:    是否自动换行
/// - font_size:    编辑器字体大小（默认 14.0）
/// - show_about:   是否显示"关于"窗口
pub struct AppState {
    pub text: String,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub font_size: f32,
    pub show_about: bool,
    fonts_initialized: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            text: String::new(),
            file_path: None,
            dirty: false,
            show_line_numbers: true,
            word_wrap: true,
            font_size: 14.0,
            show_about: false,
            fonts_initialized: false,
        }
    }
}

impl AppState {
    /// 重置所有状态来模拟"新建文件"操作。
    pub fn new_file(&mut self) {
        self.text.clear();
        self.file_path = None;
        self.dirty = false;
    }

    /// 打开文件对话框并读取内容。
    ///
    /// 使用 rfd::FileDialog 弹出原生 OS 文件选择器。
    /// 读取成功则更新文本缓冲区和文件路径，失败则打印错误并保持旧状态。
    pub fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.text = content;
                    self.file_path = Some(path);
                    self.dirty = false;
                }
                Err(err) => {
                    eprintln!("Failed to open file: {err}");
                }
            }
        }
    }

    /// 保存当前文本到关联文件路径。无路径则回退到 save_file_as。
    pub fn save_file(&mut self) {
        if let Some(ref path) = self.file_path {
            match std::fs::write(path, &self.text) {
                Ok(()) => self.dirty = false,
                Err(err) => eprintln!("Failed to save file: {err}"),
            }
        } else {
            self.save_file_as();
        }
    }

    /// 始终弹出"另存为"对话框，让用户选择保存路径。
    pub fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            match std::fs::write(&path, &self.text) {
                Ok(()) => {
                    self.file_path = Some(path);
                    self.dirty = false;
                }
                Err(err) => eprintln!("Failed to save file: {err}"),
            }
        }
    }

    /// 返回当前文件的文件名（不含路径），用于窗口标题。"新建文件"时返回 "Untitled"。
    pub fn file_name(&self) -> &str {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
    }

    /// 增大字体大小（上限 32px）。
    pub fn increase_font_size(&mut self) {
        self.font_size = (self.font_size + 1.0).min(32.0);
    }

    /// 减小字体大小（下限 8px）。
    pub fn decrease_font_size(&mut self) {
        self.font_size = (self.font_size - 1.0).max(8.0);
    }

    pub fn configure_appearance(&mut self, ctx: &egui::Context) {
        if self.fonts_initialized {
            return;
        }
        self.fonts_initialized = true;

        self.load_cjk_font(ctx);
        self.apply_zed_theme(ctx);
    }

    fn load_cjk_font(&self, ctx: &egui::Context) {
        let font_path = "/System/Library/Fonts/Hiragino Sans GB.ttc";
        let font_bytes = match std::fs::read(font_path) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("Failed to load CJK font: {err}");
                return;
            }
        };

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Hiragino".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(font_bytes).tweak(
                egui::FontTweak {
                    scale: 0.88,
                    y_offset: 2.0,
                    ..Default::default()
                },
            )),
        );
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.push("Hiragino".to_owned());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.push("Hiragino".to_owned());
        }
        ctx.set_fonts(fonts);
    }

    fn apply_zed_theme(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(0x1E, 0x1E, 0x1E);
        visuals.window_fill = egui::Color32::from_rgb(0x25, 0x25, 0x26);
        visuals.extreme_bg_color = egui::Color32::from_rgb(0x14, 0x14, 0x14);
        visuals.faint_bg_color = egui::Color32::from_rgb(0x2D, 0x2D, 0x2D);
        visuals.code_bg_color = egui::Color32::from_rgb(0x1E, 0x1E, 0x1E);
        visuals.text_edit_bg_color = Some(egui::Color32::from_rgb(0x1E, 0x1E, 0x1E));
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.window_corner_radius = egui::CornerRadius::same(4);
        visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3C, 0x3C, 0x3C));
        visuals.selection.bg_fill = egui::Color32::from_rgb(0x26, 0x4F, 0x78);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0xD4, 0xD4, 0xD4));
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3C, 0x3C, 0x3C));
        visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0x52, 0x52, 0x52));
        visuals.widgets.active.bg_stroke =
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0x00, 0x78, 0xD4));
        visuals.override_text_color = Some(egui::Color32::from_rgb(0xD4, 0xD4, 0xD4));
        ctx.set_visuals(visuals);
    }
}
