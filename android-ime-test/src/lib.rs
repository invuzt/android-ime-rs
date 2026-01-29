use android_ime::{AndroidImeConnection, AndroidImeConnectionHandler};
use anyhow::anyhow;
use eframe::egui::{Align, CentralPanel, Context, FontFamily, FontId, Layout, RichText};
use eframe::emath::Vec2;
use eframe::Frame;
use jni::objects::JObject;
use jni::JavaVM;
use log::{error, info};
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
            let (tx, rx) = std::sync::mpsc::channel();
            let handler = Handler {
                tx,
                context: cc.egui_ctx.clone(),
            };

            let mut env = java_wm.attach_current_thread()?;
            let ime_connection = AndroidImeConnection::new(&mut env, &activity, handler)?;

            cc.egui_ctx.set_zoom_factor(scale_factor);

            Ok(Box::new(MyApp {
                rx,
                log: Default::default(),
                ime_connection,
            }))
        }),
    );
    result.map_err(|e| anyhow!("{e:?}"))
}

////////////////////////////////////////////////////////////////////////////////
pub struct MyApp {
    rx: std::sync::mpsc::Receiver<String>,
    log: VecDeque<String>,
    ime_connection: AndroidImeConnection<Handler>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let _ = frame;

        while let Ok(message) = self.rx.try_recv() {
            self.log.push_front(message);
        }

        while self.log.len() > 20 {
            self.log.pop_back();
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);

                if ui.button("Show IME").clicked() {
                    if let Err(e) = self.ime_connection.activate() {
                        self.log.push_front(e.to_string());
                    }
                }

                if ui.button("Hide IME").clicked() {
                    if let Err(e) = self.ime_connection.deactivate() {
                        self.log.push_front(e.to_string());
                    }
                }

                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;

                    for message in self.log.iter() {
                        ui.label(RichText::new(message).font(FontId::new(5.0, FontFamily::Monospace)));
                    }
                });
            });
        });
    }
}

////////////////////////////////////////////////////////////////////////////////
struct Handler {
    tx: std::sync::mpsc::Sender<String>,
    context: Context,
}

impl Handler {
    fn log(&self, message: String) {
        info!("{message}");
        let _ = self.tx.send(message);
        self.context.request_repaint();
    }
}

impl AndroidImeConnectionHandler for Handler {
    fn connection_closed(&self) {
        self.log("connection_closed".to_string());
    }

    fn send_key_event(&self, key_code: i32) -> bool {
        self.log(format!("send_key_event: {key_code}"));
        true
    }

    fn perform_context_menu_action(&self, action_id: i32) -> bool {
        self.log(format!("perform_context_menu_action: {action_id}"));
        true
    }

    fn perform_editor_action(&self, editor_action: i32) -> bool {
        self.log(format!("perform_editor_action: {editor_action}"));
        true
    }

    fn commit_text(&self, text: &str, new_cursor_position: i32) -> bool {
        self.log(format!("commit_text: {text}, {new_cursor_position}"));
        true
    }

    fn delete_surrounding_text(&self, before: usize, after: usize) -> bool {
        self.log(format!("delete_surrounding_text: {before}, {after}"));
        true
    }

    fn delete_surrounding_text_in_code_points(&self, before: usize, after: usize) -> bool {
        self.log(format!("delete_surrounding_text_in_code_points: {before}, {after}"));
        true
    }

    fn set_selection(&self, start: usize, end: usize) -> bool {
        self.log(format!("set_selection: {start}, {end}"));
        true
    }

    fn set_composing_region(&self, start: usize, end: usize) -> bool {
        self.log(format!("set_composing_region: {start}, {end}"));
        true
    }

    fn set_composing_text(&self, input: &str, new_cursor_position: i32) -> bool {
        self.log(format!("set_composing_text: {input}, {new_cursor_position}"));
        true
    }

    fn finish_composing_text(&self) -> bool {
        self.log("finish_composing_text".to_string());
        true
    }

    fn get_selected_text(&self, flags: i32) -> Option<&str> {
        self.log(format!("get_selected_text: {flags}"));
        None
    }

    fn get_text_after_cursor(&self, count: usize, flags: i32) -> Option<&str> {
        self.log(format!("get_text_after_cursor: {count} {flags}"));
        None
    }

    fn get_text_before_cursor(&self, count: usize, flags: i32) -> Option<&str> {
        self.log(format!("get_text_before_cursor: {count} {flags}"));
        None
    }

    fn get_cursor_caps_mode(&self, req_modes: i32) -> i32 {
        self.log(format!("get_cursor_caps_mode: {req_modes}"));
        0
    }

    fn request_cursor_updates(&self, cursor_update_mode: i32) -> bool {
        self.log(format!("request_cursor_updates: {cursor_update_mode}"));
        true
    }
}
