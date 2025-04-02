pub mod ai;
pub mod loader;
pub mod node;

use eyre::Error;
use serde::Serialize;
use std::fmt::{Debug, Display};

pub(crate) fn send_event<S: Display, D: Serialize + Debug>(name: S, data: &D) -> Result<(), Error> {
    #[cfg(target_arch = "wasm32")]
    wasm::send_event(name, data)?;
    #[cfg(not(target_arch = "wasm32"))]
    tracing::trace!("Sending event {name} with data: {data:?}");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use eyre::{eyre, Error};
    use serde::Serialize;
    use std::fmt::Display;
    use web_sys::{window, CustomEvent};

    pub(crate) fn send_event<S: Display, V: Serialize>(tp: S, val: &V) -> Result<(), Error> {
        let window = window().ok_or_else(|| eyre!("should have a window in this context"))?;
        let tp = tp.to_string();

        let detail = serde_wasm_bindgen::to_value(val)
            .map_err(|err| eyre!("Failed to serialize event: {}", err))?;

        let evn =
            CustomEvent::new(&tp).map_err(|err| eyre!("Failed to create event: {:?}", err))?;
        evn.init_custom_event_with_can_bubble_and_cancelable_and_detail(&tp, false, false, &detail);
        window
            .dispatch_event(&evn)
            .map_err(|err| eyre!("Failed to dispatch event: {:?}", err))?;
        Ok(())
    }
}
