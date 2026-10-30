#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use std::net::TcpListener;
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

// Kicsi helyi szerver: a beépített weboldalt adja ki a 127.0.0.1 címen.
// Az ablak megnyitása elott indul, így nincs versenyhelyzet.
fn serve(app: &tauri::AppHandle) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", PORT))?;
    let resolver = app.asset_resolver();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut s = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };
            let resolver = resolver.clone();
            std::thread::spawn(move || {
                let mut buf = [0u8; 8192];
                let n = s.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    return;
                }
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let path = req
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .unwrap_or("/")
                    .split(|c| c == '?' || c == '#')
                    .next()
                    .unwrap_or("/")
                    .to_string();
                let key = if path == "/" {
                    "index.html".to_string()
                } else {
                    path.trim_start_matches('/').to_string()
                };
                match resolver.get(key) {
                    Some(a) => {
                        let head = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
                            a.mime_type,
                            a.bytes.len()
                        );
                        let _ = s.write_all(head.as_bytes());
                        let _ = s.write_all(&a.bytes);
                    }
                    None => {
                        let _ = s.write_all(
                            b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        );
                    }
                }
            });
        }
    });
    Ok(())
}

fn main() {
    tauri::Builder::default()
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
            serve(app.handle())?;
            // Média-billentyuk (ha foglaltak, nem baj, csak kihagyja)
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
