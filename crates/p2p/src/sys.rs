use futures::{channel::mpsc::Receiver, SinkExt};
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub fn now_secs() -> u64 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(target_arch = "wasm32")]
pub fn now_secs() -> u64 {
    use web_sys::window;

    (window()
        .expect("window")
        .performance()
        .expect("performance")
        .now()
        / 1000.0) as u64
}

#[inline]
pub fn is_web() -> bool {
    cfg!(target_arch = "wasm32")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interval_generator(interval: Duration) -> Receiver<()> {
    use tokio::time;

    let (tx, rx) = futures::channel::mpsc::channel(1);

    tokio::spawn(async move {
        let mut interval = time::interval(interval);
        loop {
            interval.tick().await;
            let _ = tx.clone().send(()).await;
        }
    });
    rx
}

#[cfg(target_arch = "wasm32")]
pub fn interval_generator(interval: Duration) -> Receiver<()> {
    use wasm_bindgen::{closure::Closure, JsCast};
    use wasm_bindgen_futures::spawn_local;
    use web_sys::window;

    let (tx, rx) = futures::channel::mpsc::channel(1);

    let cb = Closure::wrap(Box::new(move || {
        let mut tx = tx.clone();
        spawn_local(async move {
            let _ = tx.send(()).await;
        });
    }) as Box<dyn Fn()>);

    if let Some(window) = window() {
        window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                interval.as_millis() as i32,
            )
            .expect("Failed to set interval");
        cb.forget();
    } else {
        panic!("Failed to get window");
    }
    rx
}

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let ms = duration.as_millis() as i32;
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let closure = Closure::once_into_js(move || {
            resolve.call0(&JsValue::NULL).unwrap();
        });

        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                ms,
            )
            .unwrap();
    });

    let _ = JsFuture::from(promise).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}
