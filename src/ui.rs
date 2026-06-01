use crate::state::AppState;

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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.configure_appearance(&ctx);
        ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));
        self.handle_keyboard_shortcuts(&ctx);
        self.update_window_title(&ctx);
        self.render_menu_bar(ui);
        self.render_editor(ui);
        self.render_about_window(&ctx);
    }
}

impl AppState {
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

    fn update_window_title(&self, ctx: &egui::Context) {
        let prefix = if self.dirty { "* " } else { "" };
        let title = format!("{}{} - NoteRust", prefix, self.file_name());
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::Panel::top("menu_bar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New\tCtrl+N").clicked() {
                        self.new_file();
                        ui.close();
                    }
                    if ui.button("Open...\tCtrl+O").clicked() {
                        self.open_file();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Save\tCtrl+S").clicked() {
                        self.save_file();
                        ui.close();
                    }
                    if ui.button("Save As...\tCtrl+Shift+S").clicked() {
                        self.save_file_as();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo\tCtrl+Z").clicked() {
                        inject_key_event(&ctx, egui::Key::Z, egui::Modifiers::CTRL);
                        ui.close();
                    }
                    if ui.button("Redo\tCtrl+Y").clicked() {
                        inject_key_event(&ctx, egui::Key::Y, egui::Modifiers::CTRL);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Cut\tCtrl+X").clicked() {
                        inject_key_event(&ctx, egui::Key::X, egui::Modifiers::CTRL);
                        ui.close();
                    }
                    if ui.button("Copy\tCtrl+C").clicked() {
                        inject_key_event(&ctx, egui::Key::C, egui::Modifiers::CTRL);
                        ui.close();
                    }
                    if ui.button("Paste\tCtrl+V").clicked() {
                        inject_key_event(&ctx, egui::Key::V, egui::Modifiers::CTRL);
                        ui.close();
                    }
                    if ui.button("Delete\tDel").clicked() {
                        inject_key_event(&ctx, egui::Key::Delete, egui::Modifiers::NONE);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Select All\tCtrl+A").clicked() {
                        inject_key_event(&ctx, egui::Key::A, egui::Modifiers::CTRL);
                        ui.close();
                    }
                });

                ui.menu_button("Settings", |ui| {
                    if ui
                        .checkbox(&mut self.show_line_numbers, "Line Numbers")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.word_wrap, "Word Wrap")
                        .clicked()
                    {
                        ui.close();
                    }
                    ui.separator();
                    ui.label(format!("Font Size: {:.0}", self.font_size));
                    if ui.button("Increase\tCtrl++").clicked() {
                        self.increase_font_size();
                        ui.close();
                    }
                    if ui.button("Decrease\tCtrl+-").clicked() {
                        self.decrease_font_size();
                        ui.close();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About NoteRust").clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                });
            });
        });
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
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
                                        .color(egui::Color32::from_rgb(0x85, 0x85, 0x85)),
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
                            .code_editor()
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
