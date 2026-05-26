use crate::state::AppState;

/// 为 AppState 实现 eframe::App trait。
///
/// 这个 trait 只有一个必须实现的方法：update()。
/// egui 的渲染循环会每帧调用一次 update()：
///   1. 首先处理输入事件（键盘、鼠标）
///   2. 调用 update() 来构建 UI
///   3. 渲染结果到屏幕
///
/// 工作流程与 Zed 的 Render trait 类似——
/// 每帧都重新构建整个 UI 树，并通过脏标记优化性能。
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── 处理键盘快捷键 ──
        //
        // ctx.input_mut(|i| ...) 提供了对原始输入的可变访问。
        // consumed 字段标记按键是否已被消费，防止重复触发。
        // Ctrl+N = 新建, Ctrl+O = 打开, Ctrl+S = 保存, Ctrl+Shift+S = 另存为
        self.handle_keyboard_shortcuts(ctx);

        // ── 更新窗口标题 ──
        //
        // 窗口标题根据文件状态动态更新：
        // - "Untitled - NoteRust": 新建文件
        // - "filename.txt - NoteRust": 已打开的文件
        // - "* filename.txt - NoteRust": 有未保存的修改
        // send_viewport_cmd 通知 winit 更新实际窗口标题。
        self.update_window_title(ctx);

        // ── 顶部菜单栏 ──
        //
        // TopBottomPanel 创建一个固定在窗口顶部的面板。
        // 使用 egui::menu::bar 来获得原生的菜单栏外观。
        // 每个菜单按钮触发对应的 AppState 方法。
        self.render_menu_bar(ctx);

        // ── 中央编辑区域 ──
        //
        // CentralPanel 占据菜单栏之外的全部可用空间。
        // TextEdit::multiline 创建一个可编辑的多行文本区域——
        // 等同于记事本的编辑区域。
        // font_id(egui::FontId::monospace(14.0)) 使用等宽字体，
        // 这是文本编辑器的标准格式。
        self.render_editor(ctx);
    }
}

impl AppState {
    /// 处理全局键盘快捷键。
    ///
    /// egui 的快捷键处理需要 consume_key 来防止事件继续传播。
    /// 如果不消费，按键将被传递给下一个获得焦点的组件。
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // 遍历输入事件并收集需要消费的按键组合。
        // 不能直接在迭代过程中调用 input.consume_key()，
        // 因为 events 是不可变借用，而 consume_key 需要可变借用。
        ctx.input_mut(|input| {
            let modifiers = input.modifiers;

            // 延迟消费：先收集所有匹配的 (modifier, key) 对，循环结束后再统一消费。
            let mut keys_to_consume: Vec<(egui::Modifiers, egui::Key)> = Vec::new();

            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers: mods,
                    ..
                } = event
                {
                    if *mods == modifiers {
                        match key {
                            egui::Key::N if modifiers.ctrl => {
                                self.new_file();
                                keys_to_consume
                                    .push((egui::Modifiers::CTRL, *key));
                            }
                            egui::Key::O if modifiers.ctrl => {
                                self.open_file();
                                keys_to_consume
                                    .push((egui::Modifiers::CTRL, *key));
                            }
                            egui::Key::S
                                if modifiers.ctrl && modifiers.shift =>
                            {
                                self.save_file_as();
                                keys_to_consume.push((
                                    egui::Modifiers::CTRL
                                        | egui::Modifiers::SHIFT,
                                    *key,
                                ));
                            }
                            egui::Key::S if modifiers.ctrl => {
                                self.save_file();
                                keys_to_consume
                                    .push((egui::Modifiers::CTRL, *key));
                            }
                            _ => {}
                        }
                    }
                }
            }

            for (mods, key) in keys_to_consume {
                input.consume_key(mods, key);
            }
        });
    }

    /// 根据当前文件状态更新原生窗口标题。
    fn update_window_title(&self, ctx: &egui::Context) {
        // dirty 标志 → 在标题前面加上 "*" 前缀（例如 "* readme.txt"）
        let prefix = if self.dirty { "* " } else { "" };
        let title = format!("{}{} - NoteRust", prefix, self.file_name());
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    /// 渲染顶部菜单栏。
    ///
    /// egui::menu::bar 创建一个原生风格的菜单栏。
    /// 每个菜单按钮在被点击时触发对应操作。
    /// 所有操作不返回 Task（同步执行），这与 Zed 的异步模式不同，
    /// 但对于记事本级别的编辑器来说完全够用。
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // ── File 菜单 ──
                ui.menu_button("File", |ui| {
                    // ui.button 返回一个 Response，.clicked() 用于检测点击事件。
                    // 与 Zed 的 .on_click() 模式类似，但更为简单。
                    if ui.button("New").clicked() {
                        self.new_file();
                        ui.close_menu(); // 关闭菜单以防止阻塞 UI
                    }
                    if ui.button("Open...").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    // separator 是菜单中的分隔线，用于将不同功能分组。
                    ui.separator();
                    if ui.button("Save").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    // ctx.send_viewport_cmd 发送关闭窗口的命令。
                    // 等价于点击关闭按钮。
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });
            });
        });
    }

    /// 渲染中央编辑区域。
    ///
    /// TextEdit::multiline() 创建一个多行文本框：
    ///   - &mut self.text: 直接绑定到 AppState 的文本缓冲区。
    ///     文本框的每次修改都会立即反映到这个 String 上。
    ///     这是 egui 即时模式的核心优势——无需显式事件处理。
    ///   - desired_width(f32::INFINITY): 文本框占满整个可用宽度
    ///   - desired_rows(0): 文本框自动增长以显示所有行
    ///   - font_id(monospace): 使用等宽字体，保持文本对齐
    ///
    /// changed() 方法返回自上一帧以来文本是否发生了变化。
    /// 我们借此更新 dirty 标志，以便在标题栏显示 "*"。
    fn render_editor(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 创建一个样式化的 TextEdit 控件。
            // add 方法返回一个 InnerResponse，其中包含：
            //   - inner:  实际渲染的文本编辑框，用于获取焦点
            //   - response: 用户交互的响应，用于检测变化
            let output = ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    // 等宽字体 14px——文本编辑器的标准选择
                    .font(egui::TextStyle::Monospace)
                    // 填满所有可用宽度
                    .desired_width(f32::INFINITY)
                    // 自动垂直扩展以显示所有行（0 行 + 自动增长）
                    .desired_rows(0),
            );

            // 如果文本内容自上一帧以来发生了变化，将文件标记为脏。
            // changed() 直接调用在 add() 的返回值上——
            // egui 0.31 的 Widget::ui 返回 WidgetResponse，它实现了 Response 的
            // 所有方法（包括 changed()）。
            if output.changed() {
                self.dirty = true;
            }
        });
    }
}
