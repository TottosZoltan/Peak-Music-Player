#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

// Fix port: ezt a címet kell felvenni a Spotify dashboardban Redirect URI-ként:
// http://127.0.0.1:47893/index.html
const PORT: u16 = 47893;

// A weboldal saját gombját nyomja meg (play, prev, nextBtn)
fn press(app: &tauri::AppHandle, id: &str) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.eval(&format!(
            "var b=document.getElementById('{}');if(b)b.click();",
            id
        ));
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_localhost::Builder::new(PORT).build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    match shortcut.key {
                        Code::MediaPlayPause => press(app, "play"),
                        Code::MediaTrackNext => press(app, "nextBtn"),
                        Code::MediaTrackPrevious => press(app, "prev"),
                        _ => {}
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Média-billentyűk (ha foglaltak, nem baj, csak kihagyja)
            for k in [Code::MediaPlayPause, Code::MediaTrackNext, Code::MediaTrackPrevious] {
                let _ = app.global_shortcut().register(Shortcut::new(None, k));
            }
            let url = format!("http://127.0.0.1:{}/index.html", PORT);
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url.parse().unwrap()))
                .title("Peak Music Player")
                .inner_size(1100.0, 760.0)
                .min_inner_size(360.0, 560.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("hiba az indításkor");
}
