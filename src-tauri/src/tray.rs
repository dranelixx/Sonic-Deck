//! System tray icon and menu management

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

/// Initialize the system tray icon and menu
pub fn init<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items
    let show_hide = MenuItemBuilder::with_id("show_hide", "Show/Hide").build(app)?;
    let stop_all = MenuItemBuilder::with_id("stop_all", "Stop All Sounds").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    // Build menu with items and separator
    let menu = MenuBuilder::new(app)
        .items(&[&show_hide, &stop_all])
        .separator()
        .items(&[&quit])
        .build()?;

    // Build tray icon with menu and event handlers
    // Note: We keep the menu but handle clicks manually to distinguish left/right
    TrayIconBuilder::<R>::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false) // Disable automatic menu on left click
        .on_menu_event(move |app, event| {
            handle_tray_menu_event(app, event.id().as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            handle_tray_icon_event(tray, event);
        })
        .build(app)?;

    tracing::info!("System tray initialized");

    Ok(())
}

/// Handle tray menu item clicks
fn handle_tray_menu_event<R: Runtime>(app: &tauri::AppHandle<R>, event_id: &str) {
    match event_id {
        "show_hide" => {
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                    tracing::debug!("Window hidden from tray menu");
                } else {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    tracing::debug!("Window shown from tray menu");
                }
            }
        }
        "stop_all" => {
            // Call the stop_all_audio command
            let manager = app.state::<crate::AudioManager>();
            if let Err(e) = crate::commands::stop_all_audio(manager) {
                tracing::error!("Failed to stop all audio from tray: {}", e);
            } else {
                tracing::debug!("Stopped all audio from tray menu");
            }
        }
        "quit" => {
            tracing::info!("Quitting application from tray menu");
            app.exit(0);
        }
        _ => {
            tracing::warn!("Unknown tray menu event: {}", event_id);
        }
    }
}

/// Handle tray icon clicks (left-click toggles window visibility)
fn handle_tray_icon_event<R: Runtime>(tray: &tauri::tray::TrayIcon<R>, event: TrayIconEvent) {
    // Only log clicks, not mouse moves to reduce spam
    match &event {
        TrayIconEvent::Click { .. } | TrayIconEvent::DoubleClick { .. } => {
            tracing::info!("Tray icon event: {:?}", event);
        }
        _ => {
            // Ignore mouse moves and other events to reduce log spam
        }
    }

    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            // Left click shows window (always try to show/focus)
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
                // Always try to show and focus the window - simpler and more reliable
                if let Err(e) = window.show() {
                    tracing::error!("Failed to show window from tray click: {}", e);
                } else {
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    tracing::info!("Window restored from tray left-click");
                }
            } else {
                tracing::error!("Could not find main window");
            }
        }
        TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            // Handle double-click same as single click for now
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
                tracing::info!("Window restored from tray double-click");
            }
        }
        _ => {
            // Silently ignore other events (mouse moves, etc.) - no logging to reduce spam
        }
    }
}
