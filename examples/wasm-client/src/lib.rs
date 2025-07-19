// File: examples/wasm-client/src/lib.rs
// Complete WebAssembly client example demonstrating StormCore integration
// Shows how to create a web-based 3D virtual world client

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, document, HtmlCanvasElement, WebGlRenderingContext};
use storm_wasm::{WasmStormEngine, WasmStormConfig};
use std::cell::RefCell;
use std::rc::Rc;

// Global state for the demo application
struct AppState {
    engine: WasmStormEngine,
    canvas: HtmlCanvasElement,
    gl_context: WebGlRenderingContext,
    last_time: f64,
    running: bool,
}

thread_local! {
    static APP_STATE: RefCell<Option<AppState>> = RefCell::new(None);
}

/// Initialize the WASM client application
#[wasm_bindgen]
pub async fn init_client() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console::log_1(&"Initializing StormCore WASM client".into());

    // Get DOM elements
    let window = window().unwrap();
    let document = document().unwrap();

    let canvas = document
        .get_element_by_id("storm-canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;

    let gl_context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    // Configure canvas
    canvas.set_width(800);
    canvas.set_height(600);

    // Initialize StormCore engine
    let mut engine = WasmStormEngine::new();
    let mut config = WasmStormConfig::new();
    config.set_enable_rendering(true);
    config.set_enable_audio(true);
    config.set_enable_physics(true);
    config.set_enable_ai(true);

    engine.initialize(config).await?;

    // Create app state
    let app_state = AppState {
        engine,
        canvas,
        gl_context,
        last_time: window.performance().unwrap().now(),
        running: false,
    };

    APP_STATE.with(|state| {
        *state.borrow_mut() = Some(app_state);
    });

    // Setup UI event handlers
    setup_ui_handlers()?;

    console::log_1(&"StormCore WASM client initialized successfully".into());
    Ok(())
}

/// Setup UI event handlers for the demo
fn setup_ui_handlers() -> Result<(), JsValue> {
    let document = document().unwrap();

    // Connect button handler
    if let Some(connect_btn) = document.get_element_by_id("connect-btn") {
        let callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            wasm_bindgen_futures::spawn_local(async {
                if let Err(e) = connect_to_world().await {
                    console::error_1(&format!("Connection failed: {:?}", e).into());
                }
            });
        }) as Box<dyn FnMut(_)>);

        connect_btn.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    // Start/Stop button handler
    if let Some(start_btn) = document.get_element_by_id("start-btn") {
        let callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            toggle_engine();
        }) as Box<dyn FnMut(_)>);

        start_btn.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    Ok(())
}

/// Connect to a virtual world
async fn connect_to_world() -> Result<(), JsValue> {
    let document = document().unwrap();

    // Get world URL from input field
    let world_url = if let Some(input) = document.get_element_by_id("world-url") {
        input
            .dyn_into::<web_sys::HtmlInputElement>()?
            .value()
    } else {
        "ws://localhost:9000".to_string() // Default
    };

    // Get protocol from select field
    let protocol = if let Some(select) = document.get_element_by_id("protocol-select") {
        select
            .dyn_into::<web_sys::HtmlSelectElement>()?
            .value()
    } else {
        "opensim".to_string() // Default
    };

    APP_STATE.with(|state| {
        if let Some(ref app_state) = *state.borrow() {
            let engine = &app_state.engine;
            wasm_bindgen_futures::spawn_local(async move {
                match engine.connect_to_world(&world_url, &protocol).await {
                    Ok(_) => {
                        console::log_1(&"Connected to virtual world successfully".into());
                        update_ui_status("Connected");
                    }
                    Err(e) => {
                        console::error_1(&format!("Connection failed: {:?}", e).into());
                        update_ui_status("Connection Failed");
                    }
                }
            });
        }
    });

    Ok(())
}

/// Toggle engine running state
fn toggle_engine() {
    APP_STATE.with(|state| {
        if let Some(ref mut app_state) = *state.borrow_mut() {
            app_state.running = !app_state.running;

            if app_state.running {
                console::log_1(&"Starting engine loop".into());
                start_engine_loop();
                update_ui_status("Running");
            } else {
                console::log_1(&"Stopping engine loop".into());
                update_ui_status("Stopped");
            }
        }
    });
}

/// Start the main engine update loop
fn start_engine_loop() {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |time: f64| {
        // Update engine
        APP_STATE.with(|state| {
            if let Some(ref mut app_state) = *state.borrow_mut() {
                if !app_state.running {
                    return;
                }

                let delta_time = (time - app_state.last_time) as f32 / 1000.0;
                app_state.last_time = time;

                // Async engine update
                let engine = &app_state.engine;
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = engine.update(delta_time).await {
                        console::error_1(&format!("Engine update failed: {:?}", e).into());
                    }
                });

                // Continue loop
                request_animation_frame(f.borrow().as_ref().unwrap());
            }
        });
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

/// Request animation frame helper
fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

/// Update UI status text
fn update_ui_status(status: &str) {
    if let Some(status_elem) = document().unwrap().get_element_by_id("status") {
        status_elem.set_text_content(Some(status));
    }
}

/// Shutdown the client
#[wasm_bindgen]
pub async fn shutdown_client() -> Result<(), JsValue> {
    console::log_1(&"Shutting down StormCore WASM client".into());

    APP_STATE.with(|state| {
        if let Some(ref mut app_state) = *state.borrow_mut() {
            app_state.running = false;

            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = app_state.engine.shutdown().await {
                    console::error_1(&format!("Shutdown failed: {:?}", e).into());
                }
            });
        }
    });

    update_ui_status("Shutdown");
    Ok(())
}

/// Get engine statistics for UI display
#[wasm_bindgen]
pub fn get_engine_stats() -> Result<JsValue, JsValue> {
    APP_STATE.with(|state| {
        if let Some(ref app_state) = *state.borrow() {
            app_state.engine.get_stats()
        } else {
            Err(JsValue::from_str("Engine not initialized"))
        }
    })
}