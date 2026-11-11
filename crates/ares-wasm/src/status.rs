use ares_core::{SlicerContext, SlicingStatus};
use wasm_bindgen::JsValue;

/// Calls `on_status(percent, text)` for each slicing status. An exception thrown by the callback
/// does not stop the slice; cancellation is not ported.
pub(crate) fn report_to(on_status: &js_sys::Function) -> impl FnMut(&SlicingStatus<'_>) + '_ {
    move |status| {
        let _ = on_status.call2(
            &JsValue::NULL,
            &JsValue::from(status.percent),
            &JsValue::from_str(status.text),
        );
    }
}

/// The slice context, reporting its statuses to `report` when there is one.
pub(crate) fn context<'a>(
    seed: u32,
    report: Option<&'a mut (impl FnMut(&SlicingStatus<'_>) + 'a)>,
) -> SlicerContext<'a> {
    let mut context = SlicerContext::new(seed);
    if let Some(report) = report {
        context.set_status_callback(report);
    }
    context
}
