use crate::state::AppState;

/// 向输入事件队列注入一个键盘事件。
///
/// 用于菜单栏中的编辑操作（撤销、剪切、复制等）——
/// 点击菜单项时合成对应的快捷键事件，由 TextEdit 自动处理。
fn inject_key_event(ctx: &egui::Context, key: egui::Key, modifiers: egui::Modifiers) {
    ctx.input_mut(|i| {
        i.events.push(egui::Event::Key {
            key,
            pressed: true,
            modifiers,
            physical_key: None,
            repeat: false,
        });
    });
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard_shortcuts(ctx);
        self.update_window_title(ctx);
        self.render_menu_bar(ctx);
        self.render_editor(ctx);
        self.render_about_window(ctx);
    }
}

impl AppState {
    /// 处理全局键盘快捷键。
    ///
    /// 遍历输入事件，匹配已知的快捷键组合：
    /// Ctrl+N=新建, Ctrl+O=打开, Ctrl+S=保存, Ctrl+Shift+S=另存为,
    /// Ctrl++=字体增大, Ctrl+-=字体减小
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input_mut(|input| {
            let mut to_consume: Vec<(egui::Modifiers, egui::Key)> = Vec::new();

            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    match key {
                        egui::Key::N if modifiers.ctrl => {
                            self.new_file();
                            to_consume.push((egui::Modifiers::CTRL, *key));
                        }
                        egui::Key::O if modifiers.ctrl => {
                            self.open_file();
                            to_consume.push((egui::Modifiers::CTRL, *key));
                        }
                        egui::Key::S if modifiers.ctrl && modifiers.shift => {
                            self.save_file_as();
                            to_consume.push((
                                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                                *key,
                            ));
                        }
                        egui::Key::S if modifiers.ctrl => {
                            self.save_file();
                            to_consume.push((egui::Modifiers::CTRL, *key));
                        }
                        egui::Key::Equals if modifiers.ctrl => {
                            self.increase_font_size();
                            to_consume.push((egui::Modifiers::CTRL, *key));
                        }
                        egui::Key::Minus if modifiers.ctrl => {
                            self.decrease_font_size();
                            to_consume.push((egui::Modifiers::CTRL, *key));
                        }
                        _ => {}
                    }
                }
            }

            for (mods, key) in to_consume {
                input.consume_key(mods, key);
            }
        });
    }

    /// 根据当前文件状态更新原生窗口标题。
    fn update_window_title(&self, ctx: &egui::Context) {
        let prefix = if self.dirty { "* " } else { "" };
        let title = format!("{}{} - NoteRust", prefix, self.file_name());
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    /// 渲染顶部菜单栏。
    ///
    /// 包含四个菜单：
    /// - File：  新建、打开、保存、另存为、分隔线、退出
    /// - Edit：  撤销、重做、剪切、复制、粘贴、删除、全选
    /// - Settings：行号开关、自动换行开关、字体大小
    /// - Help：  关于
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New\tCtrl+N").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Open...\tCtrl+O").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save\tCtrl+S").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Save As...\tCtrl+Shift+S").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo\tCtrl+Z").clicked() {
                        inject_key_event(ctx, egui::Key::Z, egui::Modifiers::CTRL);
                        ui.close_menu();
                    }
                    if ui.button("Redo\tCtrl+Y").clicked() {
                        inject_key_event(ctx, egui::Key::Y, egui::Modifiers::CTRL);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Cut\tCtrl+X").clicked() {
                        inject_key_event(ctx, egui::Key::X, egui::Modifiers::CTRL);
                        ui.close_menu();
                    }
                    if ui.button("Copy\tCtrl+C").clicked() {
                        inject_key_event(ctx, egui::Key::C, egui::Modifiers::CTRL);
                        ui.close_menu();
                    }
                    if ui.button("Paste\tCtrl+V").clicked() {
                        inject_key_event(ctx, egui::Key::V, egui::Modifiers::CTRL);
                        ui.close_menu();
                    }
                    if ui.button("Delete\tDel").clicked() {
                        inject_key_event(ctx, egui::Key::Delete, egui::Modifiers::NONE);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Select All\tCtrl+A").clicked() {
                        inject_key_event(ctx, egui::Key::A, egui::Modifiers::CTRL);
                        ui.close_menu();
                    }
                });

                ui.menu_button("Settings", |ui| {
                    if ui
                        .checkbox(&mut self.show_line_numbers, "Line Numbers")
                        .clicked()
                    {
                        ui.close_menu();
                    }
                    if ui
                        .checkbox(&mut self.word_wrap, "Word Wrap")
                        .clicked()
                    {
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label(format!("Font Size: {:.0}", self.font_size));
                    if ui.button("Increase\tCtrl++").clicked() {
                        self.increase_font_size();
                        ui.close_menu();
                    }
                    if ui.button("Decrease\tCtrl+-").clicked() {
                        self.decrease_font_size();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About NoteRust").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    /// 渲染中央编辑区域，包含行号和文本编辑器。
    ///
    /// 布局结构：
    /// ┌──────────────────────────────────────┐
    /// │  ScrollArea::vertical                │
    /// │  ┌──────────────────────────────┐    │
    /// │  │  ui.horizontal (Top 对齐)     │    │
    /// │  │  ┌──────┬───┬──────────────┐ │    │
    /// │  │  │行号  │ │ │   TextEdit    │ │    │
    /// │  │  │1     │ │ │   (等宽)     │ │    │
    /// │  │  │2     │ │ │              │ │    │
    /// │  │  │3     │ │ │              │ │    │
    /// │  │  └──────┴───┴──────────────┘ │    │
    /// │  └──────────────────────────────┘    │
    /// └──────────────────────────────────────┘
    ///
    /// TextEdit 使用 desired_rows(0) 自动扩展以显示所有行，
    /// 不启用内部滚动——所有滚动由外层 ScrollArea 统一管理。
    /// 行号与文本因此始终同步滚动。
    fn render_editor(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let line_count = self.text.lines().count().max(1);
            let max_digits = format!("{}", line_count).len().max(3);
            let gutter_width = (max_digits + 1) as f32 * 10.0;

            let need_horizontal_scroll = !self.word_wrap;
            let scroll_area = if need_horizontal_scroll {
                egui::ScrollArea::both()
            } else {
                egui::ScrollArea::vertical()
            };

            scroll_area.show(ui, |ui| {
                let layout =
                    egui::Layout::left_to_right(egui::Align::Min);

                ui.with_layout(layout, |ui| {
                    if self.show_line_numbers {
                        ui.with_layout(
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_min_width(gutter_width);
                                for i in 1..=line_count {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{:>width$}",
                                            i,
                                            width = max_digits
                                        ))
                                        .font(egui::FontId::monospace(self.font_size))
                                        .color(egui::Color32::GRAY),
                                    );
                                }
                            },
                        );
                        ui.separator();
                    }

                    let desired_width = if self.word_wrap {
                        f32::INFINITY
                    } else {
                        2000.0
                    };

                    let output = ui.add(
                        egui::TextEdit::multiline(&mut self.text)
                            .font(egui::FontId::monospace(self.font_size))
                            .desired_width(desired_width)
                            .desired_rows(0),
                    );

                    if output.changed() {
                        self.dirty = true;
                    }
                });
            });
        });
    }

    /// 渲染"关于"弹窗。
    fn render_about_window(&mut self, ctx: &egui::Context) {
        let show_about = &mut self.show_about;
        egui::Window::new("About NoteRust")
            .open(show_about)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("NoteRust");
                    ui.label("A Notepad-like text editor built with egui");
                    ui.separator();
                    ui.label("Version 0.1.0");
                    ui.label("Press ESC to close");
                });
            });
    }
}
