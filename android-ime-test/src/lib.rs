use android_ime::{
    AndroidImeConnectionHandler, AndroidImeContext, AndroidImeEditable, AndroidImeEditableHandler,
};
use anyhow::anyhow;
use eframe::egui::{CentralPanel, Context, RichText, ScrollArea};
use eframe::Frame;
use jni::objects::JObject;
use jni::JavaVM;
use log::error;
use std::collections::VecDeque;
use winit::platform::android::activity::AndroidApp;

// Import widget buatan kita
mod edit_text_widget;
use edit_text_widget::EditTextWidget;

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
            struct Handler(Context);

            impl AndroidImeEditableHandler for Handler {
                fn text_updated(&self) {
                    self.0.request_repaint();
                }
            }

            let mut env = java_wm.attach_current_thread()?;
            let context = AndroidImeContext::new(&mut env, &activity)?;
            let editable = AndroidImeEditable::new(&context, Handler(cc.egui_ctx.clone()));

            cc.egui_ctx.set_zoom_factor(scale_factor);

            Ok(Box::new(MyApp {
                log: Default::default(),
                editable,
                input_field: EditTextWidget::new().with_placeholder("Ketik sesuatu di sini..."),
                output_text: String::new(),
            }))
        }),
    );
    result.map_err(|e| anyhow!("{e:?}"))
}

////////////////////////////////////////////////////////////////////////////////
pub struct MyApp {
    log: VecDeque<String>,
    editable: AndroidImeEditable,
    input_field: EditTextWidget,
    output_text: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let _ = frame;

        // Simpan teks dari input field ke output setiap saat
        self.output_text = self.input_field.text().to_string();

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);

                // INPUT FIELD - klik di sini keyboard akan muncul!
                ui.heading("📝 Input Field:");
                let response = self.input_field.show(ui, &self.editable);
                
                if response.changed() {
                    self.log.push_front(format!("Teks berubah: {}", self.input_field.text()));
                }

                ui.add_space(16.0);

                // Tombol manual (opsional)
                ui.horizontal(|ui| {
                    if ui.button("🔽 Show IME (Manual)").clicked() {
                        if let Err(e) = self.editable.show_soft_keyboard() {
                            self.log.push_front(e.to_string());
                        }
                    }
                    
                    if ui.button("🔼 Hide IME (Manual)").clicked() {
                        if let Err(e) = self.editable.hide_soft_keyboard() {
                            self.log.push_front(e.to_string());
                        }
                    }
                    
                    if ui.button("🗑 Clear").clicked() {
                        self.input_field.set_text("");
                        self.log.push_front("Input field cleared".to_string());
                    }
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Output / Log
                ui.heading("📄 Output / Log:");
                ui.add_space(8.0);
                
                ScrollArea::vertical()
                    .max_height(300.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.label(format!("✏️ Teks saat ini: \"{}\"", self.output_text));
                        ui.add_space(8.0);
                        
                        for message in self.log.iter().take(15) {
                            ui.label(RichText::new(message).small());
                        }
                        
                        if self.log.is_empty() {
                            ui.label(RichText::new("Belum ada aktivitas...").weak());
                        }
                    });
            });
        });
    }
}
