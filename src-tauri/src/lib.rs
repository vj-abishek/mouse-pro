use device_query::{DeviceQuery, DeviceState};
use std::sync::{Arc, Mutex};
use tauri::State;
use serde::Serialize;
#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Accessibility::*,
    UI::UIAutomation::*,
};

// Struct to hold the mouse position and button state
#[derive(Clone, Serialize)]
pub struct MouseState {
    position: (i32, i32),
    buttons: Vec<bool>,
    element_size: Option<(i32, i32)>, // (width, height) of clicked element
}

// Struct to hold the shared state
pub struct MouseTracker(Arc<Mutex<MouseState>>);

#[tauri::command]
fn get_mouse_state(state: State<MouseTracker>) -> MouseState {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn update_element_size(state: State<MouseTracker>, width: i32, height: i32) {
    let mut state = state.0.lock().unwrap();
    state.element_size = Some((width, height));
}

#[cfg(target_os = "windows")]
fn get_element_under_cursor() -> Option<(i32, i32)> {
    unsafe {
        // Initialize UI Automation with elevated permissions
        let automation = CoCreateInstance::<_, IUIAutomation>(
            &CUIAutomation::default(),
            None,
            CLSCTX_INPROC_SERVER,
        ).ok()?;

        // Get the cursor position
        let mut point = POINT::default();
        GetCursorPos(&mut point);

        // Get the window under the cursor
        let hwnd = WindowFromPoint(point);
        if hwnd.0 == 0 {
            return None;
        }

        // Get the element from the current mouse position
        let element = automation.ElementFromPoint(tagPOINT { 
            x: point.x, 
            y: point.y 
        }).ok()?;

        // Get the bounding rectangle
        let mut rect = tagRECT::default();
        element.get_CurrentBoundingRectangle(&mut rect).ok()?;

        Some((
            rect.right - rect.left,
            rect.bottom - rect.top
        ))
    }
}

#[cfg(not(target_os = "windows"))]
fn get_element_under_cursor() -> Option<(i32, i32)> {
    None // Placeholder for other operating systems
}

// Modify the thread that tracks mouse movement
pub fn run() {
    let mouse_state = Arc::new(Mutex::new(MouseState {
        position: (0, 0),
        buttons: vec![false; 3],
        element_size: None,
    }));
    let mouse_state_clone = mouse_state.clone();

    std::thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut last_position = (0, 0);
        let mut last_buttons = vec![];
        
        // Initialize COM for this thread
        #[cfg(target_os = "windows")]
        unsafe {
            CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED).ok();
        }
        
        loop {
            let mouse = device_state.get_mouse();
            let current_position = mouse.coords;
            let current_buttons = mouse.button_pressed;
            
            if current_position != last_position || current_buttons != last_buttons {
                let mut state = mouse_state_clone.lock().unwrap();
                state.position = current_position;
                state.buttons = current_buttons.clone();

                // Update element size when clicking
                if !last_buttons.is_empty() && current_buttons.contains(&true) && !last_buttons.contains(&true) {
                    if let Some(size) = get_element_under_cursor() {
                        state.element_size = Some(size);
                    }
                }

                last_position = current_position;
                last_buttons = current_buttons;
            }
            
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    tauri::Builder::default()
        .manage(MouseTracker(mouse_state))
        .invoke_handler(tauri::generate_handler![get_mouse_state, update_element_size])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}