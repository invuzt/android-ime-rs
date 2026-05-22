use android_ime::{
    AndroidImeContext, AndroidImeEditable, AndroidImeEditableHandler,
};
use anyhow::anyhow;
use eframe::egui::{CentralPanel, Context, RichText, ScrollArea, TextEdit};
use eframe::Frame;
use jni::objects::JObject;
use jni::JavaVM;
use log::error;
use std::collections::VecDeque;
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

    let result = eframe::run_native(
        "AndroidIme Test App",
        options,
        Box::new(move |cc| {
            // Handler untuk menerima update dari IME
            struct Handler {
                ctx: Context,
                app_ptr: *mut MyApp, // Pointer ke MyApp untuk update teks
            }
            unsafe impl Send for Handler {}
            
            impl AndroidImeEditableHandler for Handler {
                fn text_updated(&self) {
                    // Request repaint, nanti di update() kita sync teks
                    self.ctx.request_repaint();
                }
            }

            let mut env = java_wm.attach_current_thread()?;
            let context = AndroidImeContext::new(&mut env, &activity)?;
            
            // Buat dulu app kosong
            let mut app = MyApp {
                log: VecDeque::new(),
                editable: None,
                input_text: String::new(),
                input_focused: false,
            };
            
            // Buat editable dengan handler
            let handler = Handler {
                ctx: cc.egui_ctx.clone(),
                app_ptr: &mut app as *mut MyApp,
            };
            let editable = AndroidImeEditable::new(&context, handler);
            app.editable = Some(editable);

            cc.egui_ctx.set_zoom_factor(scale_factor);

            Ok(Box::new(app))
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
}

impl MyApp {
    fn sync_text_from_ime(&mut self) {
        if let Some(editable) = &self.editable {
            let new_text = editable.content().text.to_string();
            if self.input_text != new_text {
                self.input_text = new_text;
                self.log.push_front("📥 Teks update dari IME".to_string());
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let _ = frame;
        
        // Sinkronkan teks dari IME setiap frame
        self.sync_text_from_ime();

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);

                ui.heading("📝 Input Field (klik untuk mengetik):");
                ui.add_space(8.0);

                // TextEdit input field
                let text_edit = TextEdit::singleline(&mut self.input_text)
                    .hint_text("Ketik di sini...");
                let response = ui.add(text_edit);

                // Handle focus: munculkan keyboard saat diklik
                if response.clicked() && !self.input_focused {
                    self.input_focused = true;
                    if let Some(editable) = &mut self.editable {
                        let _ = editable.show_soft_keyboard();
                    }
                    self.log.push_front("🔽 Keyboard muncul".to_string());
                }

                // Handle lost focus: sembunyikan keyboard
                if response.lost_focus() && self.input_focused {
                    self.input_focused = false;
                    if let Some(editable) = &mut self.editable {
                        let _ = editable.hide_soft_keyboard();
                    }
                    self.log.push_front("🔼 Keyboard sembunyi".to_string());
                }

                // Kirim teks ke IME saat user mengetik langsung
                if response.changed() {
                    if let Some(editable) = &self.editable {
                        // Update teks di IME
                        let _ = editable.set_text(&self.input_text);
                    }
                    self.log.push_front(format!("✏️ Teks berubah: {}", self.input_text));
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Tombol manual (opsional)
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
                        if let Some(editable) = &self.editable {
                            let _ = editable.set_text("");
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
