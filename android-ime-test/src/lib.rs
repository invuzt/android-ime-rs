use android_ime::{
    AndroidImeContext, AndroidImeEditable, AndroidImeEditableHandler,
};
use anyhow::anyhow;
use eframe::egui::{CentralPanel, Context, RichText, ScrollArea, TextEdit};
use eframe::Frame;
use jni::objects::JObject;
use jni::JavaVM;
use log::{error, info};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use winit::platform::android::activity::AndroidApp;

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("RUST_ANDROID_IME")
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

    let result = eframe::run_native(
        "AndroidIme Test App",
        options,
        Box::new(move |cc| {
            struct Handler {
                ctx: Context,
                shared_text: Arc<Mutex<String>>,
            }

            impl AndroidImeEditableHandler for Handler {
                fn text_updated(&self) {
                    info!(">>> text_updated() CALLED! <<<");
                    self.ctx.request_repaint();
                }
                
                fn commit_text(&self, text: &str, new_cursor_position: isize) -> bool {
                    info!(">>> commit_text: '{}', cursor: {} <<<", text, new_cursor_position);
                    if let Ok(mut shared) = self.shared_text.lock() {
                        *shared = text.to_string();
                        info!(">>> shared_text sekarang: '{}' <<<", shared);
                    }
                    self.ctx.request_repaint();
                    true
                }
                
                fn set_composing_text(&self, input: &str, new_cursor_position: isize) -> bool {
                    info!(">>> set_composing_text: '{}' <<<", input);
                    if let Ok(mut shared) = self.shared_text.lock() {
                        *shared = input.to_string();
                    }
                    self.ctx.request_repaint();
                    true
                }
                
                fn delete_surrounding_text(&self, before: usize, after: usize) -> bool {
                    info!(">>> delete_surrounding_text: before={}, after={} <<<", before, after);
                    if let Ok(mut shared) = self.shared_text.lock() {
                        let len = shared.len();
                        if before <= len {
                            shared.truncate(len - before);
                        }
                    }
                    self.ctx.request_repaint();
                    true
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
                input_text: String::new(),
                input_focused: false,
                shared_text: shared_text.clone(),
            }))
        }),
    );
    result.map_err(|e| anyhow!("{e:?}"))
}

////////////////////////////////////////////////////////////////////////////////
pub struct MyApp {
    log: VecDeque<String>,
    editable: Option<AndroidImeEditable>,
    input_text: String,
    input_focused: bool,
    shared_text: Arc<Mutex<String>>,
}

impl MyApp {
    fn sync_text_from_ime(&mut self) {
        if let Ok(shared) = self.shared_text.lock() {
            if self.input_text != *shared {
                self.input_text = shared.clone();
                self.log.push_front(format!("📥 Teks dari IME: '{}'", self.input_text));
                info!(">>> UI sync: input_text = '{}' <<<", self.input_text);
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let _ = frame;
        
        self.sync_text_from_ime();

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);

                ui.heading("📝 Input Field (klik untuk mengetik):");
                ui.add_space(8.0);

                let text_edit = TextEdit::singleline(&mut self.input_text)
                    .hint_text("Ketik di sini...");
                let response = ui.add(text_edit);

                if response.clicked() && !self.input_focused {
                    self.input_focused = true;
                    if let Some(editable) = &mut self.editable {
                        let _ = editable.show_soft_keyboard();
                    }
                    self.log.push_front("🔽 Keyboard muncul".to_string());
                    info!(">>> Keyboard shown <<<");
                }

                if response.lost_focus() && self.input_focused {
                    self.input_focused = false;
                    if let Some(editable) = &mut self.editable {
                        let _ = editable.hide_soft_keyboard();
                    }
                    self.log.push_front("🔼 Keyboard sembunyi".to_string());
                }

                if response.changed() {
                    // Update shared_text saat user ngetik via keyboard fisik (bukan IME)
                    if let Ok(mut shared) = self.shared_text.lock() {
                        *shared = self.input_text.clone();
                    }
                    self.log.push_front(format!("✏️ Teks berubah: {}", self.input_text));
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("🔽 Show IME").clicked() {
                        if let Some(editable) = &mut self.editable {
                            let _ = editable.show_soft_keyboard();
                        }
                        self.input_focused = true;
                    }
                    if ui.button("🔼 Hide IME").clicked() {
                        if let Some(editable) = &mut self.editable {
                            let _ = editable.hide_soft_keyboard();
                        }
                        self.input_focused = false;
                    }
                    if ui.button("🗑 Clear").clicked() {
                        self.input_text.clear();
                        if let Ok(mut shared) = self.shared_text.lock() {
                            shared.clear();
                        }
                        self.log.push_front("Cleared".to_string());
                    }
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.heading("📄 Log:");
                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        ui.label(format!("📌 Teks saat ini: \"{}\"", self.input_text));
                        ui.add_space(8.0);
                        for msg in self.log.iter().take(15) {
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
