use leptos::{html::AnyElement, *};

use crate::{components::primitive::Primitive, Attributes};

const DEFAULT_MAX: f64 = 100.0;

#[derive(Clone)]
struct ProgressContextValue {
  value: Signal<Option<f64>>,
  max: Signal<f64>,
}

#[component]
pub fn ProgressRoot(
  #[prop(optional, into)] value: MaybeProp<f64>,
  #[prop(default=100.0f64.into(), into)] max: MaybeSignal<f64>,
  #[prop(optional)] get_value_label: Option<Callback<(f64, f64), String>>,
  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: Children,
) -> impl IntoView {
  let max = Signal::derive(move || {
    let max = max.get();

    if !max.is_nan() && max > 0.0 {
      max
    } else {
      DEFAULT_MAX
    }
  });

  let value = Signal::derive(move || {
    let max = max.get();

    value
      .get()
      .and_then(|value| (!value.is_nan() && value <= max && value >= 0.0).then_some(value))
  });

  let get_value_label = get_value_label.unwrap_or(Callback::new(|(value, max): (f64, f64)| {
    format!("{}%", (value / max).round() * 100.0)
  }));

  let value_label = Signal::derive(move || {
    value
      .get()
      .map(|value| get_value_label.call((value, max.get())))
  });

  provide_context(ProgressContextValue {
    value,
    max: Signal::derive(move || max.get()),
  });

  let mut merged_attrs = attrs.clone();

  merged_attrs.extend([
    ("role", "progressbar".into_attribute()),
    ("aria-valuemax", max.into_attribute()),
    ("aria-valuemin", 0.into_attribute()),
    ("aria-valuenow", value.into_attribute()),
    ("aria-valuetext", value_label.into_attribute()),
    (
      "data-state",
      (move || {
        value
          .get()
          .map(|value| {
            if value == max.get() {
              "complete"
            } else {
              "loading"
            }
          })
          .unwrap_or("indeterminate")
      })
      .into_attribute(),
    ),
    ("data-value", value.into_attribute()),
    ("data-max", max.into_attribute()),
  ]);

  view! {
    <Primitive
      element=html::div
      node_ref=node_ref
      attrs=merged_attrs
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn ProgressIndicator(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
) -> impl IntoView {
  let ProgressContextValue { max, value } =
    use_context().expect("ProgressIndicator needs to be in a Progress component");

  let mut merged_attrs = attrs.clone();

  merged_attrs.extend([
    (
      "data-state",
      (move || {
        value
          .get()
          .map(|value| {
            if value == max.get() {
              "complete"
            } else {
              "loading"
            }
          })
          .unwrap_or("indeterminate")
      })
      .into_attribute(),
    ),
    ("data-value", value.into_attribute()),
    ("data-max", max.into_attribute()),
  ]);

  view! {
    <Primitive
      element=html::div
      node_ref=node_ref
      attrs=merged_attrs
    >
      {().into_view()}
    </Primitive>
  }
}
