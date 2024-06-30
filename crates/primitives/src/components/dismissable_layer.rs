use ev::{FocusEvent, KeyboardEvent, PointerEvent};
use html::AnyElement;
use itertools::Either;
use leptos::*;

use crate::{primitive::Primitive, util::Attributes};

#[derive(Clone)]
pub struct DismissableLayerContext {
  layers: Signal<Vec<NodeRef<AnyElement>>>,
  layers_with_outside_pointer_events_disabled: Signal<Vec<NodeRef<AnyElement>>>,
  branches: Signal<Vec<NodeRef<AnyElement>>>,
}

#[component]
pub fn DismissableLayer(
  #[prop(optional, into)] disable_outside_pointer_events: MaybeProp<bool>,

  #[prop(default=(|_|{}).into(), into)] on_escape_key_down: Callback<KeyboardEvent>,
  #[prop(default=(|_|{}).into(), into)] on_pointer_down_outside: Callback<PointerEvent>,
  #[prop(default=(|_|{}).into(), into)] on_focus_outside: Callback<FocusEvent>, // PointerDownOutsideEvent
  #[prop(default=(|_|{}).into(), into)] on_interact_outside: Callback<
    Either<PointerEvent, FocusEvent>,
  >, // FocusOutsideEvent
  #[prop(default=(|_|{}).into(), into)] on_dismiss: Callback<()>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let DismissableLayerContext {
    layers,
    layers_with_outside_pointer_events_disabled,
    branches,
  } = use_context::<DismissableLayerContext>()
    .expect("DismissableLayer must be used in a component that provides DismissableLayerContext");

  let owner_document = Signal::derive(move || {
    if let Some(node) = node_ref.get() {
      node.owner_document().unwrap_or(document())
    } else {
      document()
    }
  });

  let index = Signal::derive(move || {
    layers.with(|layers| {
      layers.iter().position(|item| {
        let Some(item_node) = item.get() else {
          return false;
        };

        let Some(node) = node_ref.get() else {
          return false;
        };

        item_node.eq(&node)
      })
    })
  });

  let highest_layer_with_outside_pointer_events_disabled = Signal::derive(move || {
    layers_with_outside_pointer_events_disabled.with(|layers| layers.last().cloned())
  });

  let highest_layer_with_outside_pointer_events_disabled_index = Signal::derive(move || {
    let layer = highest_layer_with_outside_pointer_events_disabled
      .get()
      .and_then(|layer| layer.get())?;

    layers.with(|layers| {
      layers.iter().position(|item| {
        let Some(item_node) = item.get() else {
          return false;
        };

        item_node.eq(&layer)
      })
    })
  });

  let is_body_pointer_events_disabled = Signal::derive(move || {
    layers_with_outside_pointer_events_disabled.with(|layers| !layers.is_empty())
  });
  let is_pointer_events_enabled = Signal::derive(move || {
    let (Some(index), Some(highest_layer_index)) = (
      index.get(),
      highest_layer_with_outside_pointer_events_disabled_index.get(),
    ) else {
      return false;
    };

    index >= highest_layer_index
  });

  Effect::new(move |_| {
    let Some(node) = node_ref.get() else {
      return;
    };

    _ = node.style("pointer-events", move || {
      is_body_pointer_events_disabled
        .get()
        .then_some(if is_pointer_events_enabled.get() {
          "auto"
        } else {
          "none"
        })
    });
  });

  view! {
    <Primitive
      attrs=attrs
      element=html::div

      on:focus=move |_| {

      }
      on:blur=move|_| {

      }
      on:pointerdown=move|_| {

      }

      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}
