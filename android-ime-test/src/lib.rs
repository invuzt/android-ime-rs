use android_ime::{
    AndroidImeContext, AndroidImeEditable, AndroidImeEditableHandler,
};
use anyhow::anyhow;
use eframe::egui::{self, CentralPanel, Color32, Context, RichText, ScrollArea, TopBottomPanel};
use eframe::Frame;
use jni::objects::JObject;
use jni::JavaVM;
use log::error;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use winit::platform::android::activity::AndroidApp;

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("CRUD_IME")
            .with_max_level(log::LevelFilter::Debug),
    );

    if let Err(e) = try_android_main(android_app) {
        error!("android_main failed: {e:?}");
    }
}

////////////////////////////////////////////////////////////////////////////////
fn try_android_main(android_app: AndroidApp) -> anyhow::Result<()> {
    let java_wm = unsafe { JavaVM::from_raw(android_app.vm_as_ptr().cast())? };
    let activity = unsafe { JObject::from_raw(android_app.activity_as_ptr().cast()) };

    let scale_factor = match android_app.config().density() {
        None => 1.0,
        Some(e) => e as f32 / 160.0,
    };
    let options = eframe::NativeOptions {
        android_app: Some(android_app),
        ..Default::default()
    };

    let shared_text = Arc::new(Mutex::new(String::new()));
    let pending_action = Arc::new(Mutex::new(PendingAction::None));
    let items = Arc::new(Mutex::new(Vec::<String>::new()));

    let result = eframe::run_native(
        "CRUD App - IME Test",
        options,
        Box::new(move |cc| {
            struct Handler {
                ctx: Context,
                shared_text: Arc<Mutex<String>>,
            }

            impl AndroidImeEditableHandler for Handler {
                fn text_updated(&self) {
                    self.ctx.request_repaint();
                }
            }

            let mut env = java_wm.attach_current_thread()?;
            let context = AndroidImeContext::new(&mut env, &activity)?;
            
            let handler = Handler {
                ctx: cc.egui_ctx.clone(),
                shared_text: shared_text.clone(),
            };
            let editable = AndroidImeEditable::new(&context, handler);

            cc.egui_ctx.set_zoom_factor(scale_factor);

            Ok(Box::new(MyApp {
                log: VecDeque::new(),
                editable: Some(editable),
                shared_text: shared_text.clone(),
                pending_action: pending_action.clone(),
                items: items.clone(),
                input_buffer: String::new(),
            }))
        }),
    );
    result.map_err(|e| anyhow!("{e:?}"))
}

#[derive(Clone, PartialEq)]
enum PendingAction {
    None,
    Add,
    Edit(usize),
}

////////////////////////////////////////////////////////////////////////////////
pub struct MyApp {
    log: VecDeque<String>,
    editable: Option<AndroidImeEditable>,
    shared_text: Arc<Mutex<String>>,
    pending_action: Arc<Mutex<PendingAction>>,
    items: Arc<Mutex<Vec<String>>>,
    input_buffer: String,
}

impl MyApp {
    fn sync_text_from_ime(&mut self) {
        if let Ok(shared) = self.shared_text.lock() {
            if self.input_buffer != *shared {
                self.input_buffer = shared.clone();
                self.log.push_front(format!("📥 Input: '{}'", self.input_buffer));
            }
        }
    }
    
    fn clear_input(&mut self) {
        self.input_buffer.clear();
        if let Ok(mut shared) = self.shared_text.lock() {
            shared.clear();
        }
    }
    
    fn add_item(&mut self) {
        if !self.input_buffer.is_empty() {
            let text = self.input_buffer.clone();
            if let Ok(mut items) = self.items.lock() {
                items.push(text.clone());
                self.log.push_front(format!("✅ Added: '{}'", text));
            }
            self.clear_input();
        }
    }
    
    fn update_item(&mut self, index: usize) {
        if !self.input_buffer.is_empty() {
            let text = self.input_buffer.clone();
            if let Ok(mut items) = self.items.lock() {
                if index < items.len() {
                    let old = items[index].clone();
                    items[index] = text;
                    self.log.push_front(format!("✏️ Updated: '{}' -> '{}'", old, items[index]));
                }
            }
            self.clear_input();
        }
    }
    
    fn delete_item(&mut self, index: usize) {
        let removed = {
            if let Ok(mut items) = self.items.lock() {
                if index < items.len() {
                    Some(items.remove(index))
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(removed) = removed {
            self.log.push_front(format!("🗑️ Deleted: '{}'", removed));
        }
    }
    
    fn show_keyboard(&mut self) {
        if let Some(editable) = &mut self.editable {
            let _ = editable.show_soft_keyboard();
            self.log.push_front("🔽 Keyboard muncul".to_string());
        }
    }
    
    fn hide_keyboard(&mut self) {
        if let Some(editable) = &mut self.editable {
            let _ = editable.hide_soft_keyboard();
            self.log.push_front("🔼 Keyboard sembunyi".to_string());
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let _ = frame;
        
        self.sync_text_from_ime();

        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.heading(RichText::new("📱 CRUD App - Keyboard Input").size(20.0));
                ui.label("Tanpa TextEdit - Input dari keyboard langsung");
            });
            ui.add_space(8.0);
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(8.0);
                
                // Current input display
                ui.group(|ui| {
                    ui.label(RichText::new("📝 Input saat ini:").strong());
                    ui.add_space(4.0);
                    let text_color = if self.input_buffer.is_empty() { Color32::GRAY } else { Color32::GREEN };
                    ui.colored_label(text_color, if self.input_buffer.is_empty() { "(belum ada input)" } else { &self.input_buffer });
                });
                
                ui.add_space(8.0);
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("➕ ADD").size(18.0)).clicked() {
                        self.clear_input();
                        if let Ok(mut action) = self.pending_action.lock() {
                            *action = PendingAction::Add;
                        }
                        self.show_keyboard();
                        self.log.push_front("📝 Mode: ADD - ketik lalu tekan Save".to_string());
                    }
                    
                    if ui.button(RichText::new("💾 SAVE").size(18.0)).clicked() {
                        let action = self.pending_action.lock().unwrap().clone();
                        match action {
                            PendingAction::Add => {
                                self.add_item();
                            }
                            PendingAction::Edit(idx) => {
                                self.update_item(idx);
                            }
                            PendingAction::None => {
                                self.log.push_front("⚠️ Tidak ada action pending. Klik Add atau Edit dulu.".to_string());
                            }
                        }
                        if let Ok(mut action) = self.pending_action.lock() {
                            *action = PendingAction::None;
                        }
                        self.hide_keyboard();
                    }
                    
                    if ui.button(RichText::new("❌ CANCEL").size(18.0)).clicked() {
                        if let Ok(mut action) = self.pending_action.lock() {
                            *action = PendingAction::None;
                        }
                        self.clear_input();
                        self.hide_keyboard();
                        self.log.push_front("❌ Dibatalin".to_string());
                    }
                });
                
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                
                // List items (CRUD display)
                ui.heading(RichText::new("📋 Data List").size(18.0));
                ui.add_space(8.0);
                
                let items_clone = {
                    if let Ok(items) = self.items.lock() {
                        items.clone()
                    } else {
                        Vec::new()
                    }
                };
                
                if items_clone.is_empty() {
                    ui.label(RichText::new("  (kosong)").weak());
                } else {
                    ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for (i, item) in items_clone.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{}", i + 1)).monospace());
                                    ui.label(RichText::new(item).strong());
                                    ui.add_space(10.0);
                                    
                                    if ui.button("✏️ Edit").clicked() {
                                        self.input_buffer = item.clone();
                                        if let Ok(mut shared) = self.shared_text.lock() {
                                            *shared = item.clone();
                                        }
                                        if let Ok(mut action) = self.pending_action.lock() {
                                            *action = PendingAction::Edit(i);
                                        }
                                        self.show_keyboard();
                                        self.log.push_front(format!("✏️ Mode: EDIT '{}' - ketik lalu tekan Save", item));
                                    }
                                    
                                    if ui.button("🗑️ Delete").clicked() {
                                        self.delete_item(i);
                                    }
                                });
                                ui.add_space(4.0);
                            }
                        });
                }
                
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                
                // Log area
                ui.heading("📄 Event Log:");
                ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for msg in self.log.iter().take(10) {
                            ui.label(RichText::new(msg).small());
                        }
                        if self.log.is_empty() {
                            ui.label(RichText::new("Belum ada aktivitas...").weak());
                        }
                    });
            });
        });
    }
}
