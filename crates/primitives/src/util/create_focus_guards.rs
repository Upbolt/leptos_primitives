use leptos::*;

use leptos_use::use_document;

use wasm_bindgen::JsCast;

pub(crate) fn create_focus_guards() {
  Effect::new(move |prev_count: Option<StoredValue<u32>>| {
    let document = use_document();
    let Some(document) = document.as_ref() else {
      return StoredValue::default();
    };

    let Ok(edge_guards) = document.query_selector_all("[data-leptix-focus-guard]") else {
      return StoredValue::default();
    };

    let Some(body) = document.body() else {
      return StoredValue::default();
    };

    _ = body.insert_adjacent_element(
      "afterbegin",
      &edge_guards
        .get(0)
        .and_then(|node| node.dyn_ref::<web_sys::Element>().cloned())
        .unwrap_or_else(create_focus_guard),
    );
    _ = body.insert_adjacent_element(
      "beforeend",
      &edge_guards
        .get(1)
        .and_then(|node| node.dyn_ref::<web_sys::Element>().cloned())
        .unwrap_or_else(create_focus_guard),
    );

    let count = prev_count
      .map(|prev_count| {
        prev_count.update_value(|prev_count| *prev_count += 1);
        prev_count
      })
      // TODO: this will probably cause issues in the cleanup function, should rethink design of this effect lifecycle
      .unwrap_or_else(|| StoredValue::new(1));

    on_cleanup(move || {
      if prev_count
        .map(|prev_count| prev_count.get_value())
        .unwrap_or_default()
        == 1
      {
        let document = use_document();
        let Some(document) = document.as_ref() else {
          return;
        };
        let Ok(edge_guards) = document.query_selector_all("[data-leptix-focus-guard]") else {
          return;
        };

        let guard_count = edge_guards.length();

        for i in 0..guard_count {
          if let Some(node) = edge_guards.get(i) {
            if let Some(node) = node.dyn_ref::<web_sys::Element>() {
              node.remove();
            }
          }
        }
      }

      if let Some(prev_count) = prev_count {
        prev_count.update_value(|prev_count| *prev_count -= 1);
      }
    });

    count
  });
}

fn create_focus_guard() -> web_sys::Element {
  html::span()
    .attrs([
      ("data-leptix-focus-guard", "".into_attribute()),
      ("tabindex", 0.into_attribute()),
      (
        "style",
        "outline: none; opacity: 0; position: fixed; pointer-events: none".into_attribute(),
      ),
    ])
    .dyn_ref::<web_sys::Element>()
    .cloned()
    .expect("failed to create focus guard")
}
