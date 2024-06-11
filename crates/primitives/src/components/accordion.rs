use std::collections::HashMap;

use leptos::{html::AnyElement, *};
use web_sys::KeyboardEvent;

use wasm_bindgen::JsCast;

use crate::{
  components::{
    collapsible::{CollapsibleContent, CollapsibleRoot, CollapsibleTrigger},
    collection::{use_collection_context, use_collection_item_ref, CollectionItemId},
    primitive::Primitive,
  },
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    create_id::create_id,
    Direction, Orientation,
  },
  Attributes,
};

use super::collection::CollectionContextValue;

pub enum AccordionKind {
  Single {
    value: MaybeProp<String>,
    default_value: MaybeProp<String>,
    on_value_change: Option<Callback<String>>,
    collapsible: MaybeSignal<bool>,
  },
  Multiple {
    value: MaybeProp<Vec<String>>,
    default_value: MaybeProp<Vec<String>>,
    on_value_change: Option<Callback<Vec<String>>>,
  },
}

pub struct AccordionSingle;
pub struct AccordionMultiple;

impl AccordionSingle {
  pub fn none() -> Option<String> {
    None
  }
}

impl AccordionMultiple {
  pub fn none() -> Option<Vec<String>> {
    None
  }
}

#[derive(Clone)]
struct AccordionContextValue {
  value: Signal<Vec<String>>,
  on_item_open: Callback<String>,
  on_item_close: Callback<String>,
}

#[derive(Clone)]
struct AccordionCollapsibleContextValue {
  collapsible: MaybeSignal<bool>,
}

#[component]
pub fn AccordionRoot(
  kind: AccordionKind,

  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(optional, into)] direction: MaybeSignal<Direction>,
  #[prop(default=Orientation::Vertical.into(), into)] orientation: MaybeSignal<Orientation>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  provide_context(
    CollectionContextValue::<AccordionCollectionItem, AnyElement> {
      collection_ref: node_ref,
      item_map: RwSignal::new(HashMap::new()),
    },
  );

  match kind {
    AccordionKind::Single {
      value,
      default_value,
      on_value_change,
      collapsible,
    } => view! {
      <AccordionSingleImpl
        value=value
        default_value=default_value
        on_value_change=on_value_change.unwrap_or((|_|{}).into())
        collapsible=Signal::derive(move || collapsible.get())
        disabled=Signal::derive(move || disabled.get())
        direction=Signal::derive(move || direction.get())
        orientation=Signal::derive(move || orientation.get())
        node_ref=node_ref
        attrs=attrs
        as_child=as_child
      >
        {children()}
      </AccordionSingleImpl>
    },
    AccordionKind::Multiple {
      value,
      default_value,
      on_value_change,
    } => view! {
      <AccordionMultipleImpl
        value=value
        default_value=default_value
        on_value_change=on_value_change.unwrap_or((|_|{}).into())
        disabled=Signal::derive(move || disabled.get())
        direction=Signal::derive(move || direction.get())
        orientation=Signal::derive(move || orientation.get())
        node_ref=node_ref
        attrs=attrs
        as_child=as_child
      >
        {children()}
      </AccordionMultipleImpl>
    },
  }
}

#[component]
fn AccordionSingleImpl(
  #[prop(optional, into)] value: MaybeProp<String>,
  #[prop(optional, into)] default_value: MaybeProp<String>,

  on_value_change: Callback<String>,

  collapsible: Signal<bool>,
  disabled: Signal<bool>,
  direction: Signal<Direction>,
  orientation: Signal<Orientation>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let (value, set_value) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || value.get()),
    default_value: Signal::derive(move || default_value.get()),
    on_change: Callback::new(move |value| {
      on_value_change.call(value);
    }),
  });

  let set_on_item_open = set_value.clone();
  let set_on_item_close = set_value.clone();

  provide_context(AccordionContextValue {
    value: Signal::derive(move || value.get().map(|value| vec![value]).unwrap_or_default()),
    on_item_open: Callback::new(move |value| {
      set_on_item_open.set(value);
    }),
    on_item_close: Callback::new(move |_| {
      if collapsible.get() {
        set_on_item_close.set(String::new());
      }
    }),
  });

  provide_context(AccordionCollapsibleContextValue {
    collapsible: Signal::derive(move || collapsible.get()).into(),
  });

  view! {
    <Accordion
      disabled=disabled
      direction=direction
      orientation=orientation
      node_ref=node_ref
      attrs=attrs
      as_child=as_child
    >
      {children()}
    </Accordion>
  }
}

#[component]
fn AccordionMultipleImpl(
  #[prop(optional)] value: MaybeProp<Vec<String>>,
  #[prop(optional)] default_value: MaybeProp<Vec<String>>,

  on_value_change: Callback<Vec<String>>,

  disabled: Signal<bool>,
  direction: Signal<Direction>,
  orientation: Signal<Orientation>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let (value, set_value) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || value.get()),
    default_value: Signal::derive(move || default_value.get()),
    on_change: Callback::new(move |value| {
      on_value_change.call(value);
    }),
  });

  let set_on_item_open = set_value.clone();
  let set_on_item_close = set_value.clone();

  provide_context(AccordionContextValue {
    value: Signal::derive(move || value.get().unwrap_or_default()),
    on_item_open: Callback::new(move |value| {
      set_on_item_open.update(|prev| {
        if let Some(prev) = prev {
          prev.push(value);
        } else {
          *prev = Some(vec![]);
        }
      });
    }),
    on_item_close: Callback::new(move |value| {
      set_on_item_close.update(|prev| {
        if let Some(prev) = prev {
          if let Some(position) = prev.iter().position(|prev_value| prev_value.eq(&value)) {
            prev.remove(position);
          }
        } else {
          *prev = Some(vec![]);
        }
      });
    }),
  });

  provide_context(AccordionCollapsibleContextValue {
    collapsible: true.into(),
  });

  view! {
    <Accordion
      disabled=disabled
      direction=direction
      orientation=orientation
      node_ref=node_ref
      attrs=attrs
      as_child=as_child
    >
      {children()}
    </Accordion>
  }
}

#[derive(Clone)]
struct AccordionStateContextValue {
  disabled: Signal<bool>,
  orientation: Signal<Orientation>,
  direction: Signal<Direction>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct AccordionCollectionItem;

#[component]
fn Accordion(
  disabled: Signal<bool>,
  orientation: Signal<Orientation>,
  direction: Signal<Direction>,
  #[prop(default=(|_|{}).into(), into)] on_key_down: Callback<KeyboardEvent>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let get_items = use_collection_context::<AccordionCollectionItem, AnyElement>();

  let is_direction_left_to_right =
    Signal::derive(move || direction.get() == Direction::LeftToRight);

  provide_context(AccordionStateContextValue {
    disabled: Signal::derive(move || disabled.get()),
    orientation: Signal::derive(move || orientation.get()),
    direction: Signal::derive(move || direction.get()),
  });

  view! {
    <Primitive
      {..attrs}
      attr:data-orientation=move || orientation.get().to_string()
      element=html::div
      on:keydown=move |ev: KeyboardEvent| {
        on_key_down.call(ev.clone());

        if !disabled.get() {
          return;
        }

        (|| {
          let target = ev.target()?;
          let target_el = target.dyn_ref::<web_sys::HtmlButtonElement>()?;
          let items = get_items.get();

          let triggers = items.iter().filter_map(|(node, _)| {
            let node = node.get()?;
            let node = node.dyn_ref::<web_sys::HtmlButtonElement>()?;

            Some((!node.disabled()).then_some(node.clone())).flatten()
          }).collect::<Vec<_>>();

          let trigger_index = triggers.iter().position(|item| item.eq(target_el))?;
          let trigger_count = triggers.len();

          ev.prevent_default();

          let mut next_index = trigger_index;

          let home_index = 0;
          let end_index = 0;

          match ev.key().as_str() {
            "Home" => {
              next_index = home_index;
            }
            "End" => {
              next_index = end_index;
            }
            "ArrowRight" => {
              if orientation.get() == Orientation::Horizontal {
                if is_direction_left_to_right.get() {
                  next_index = trigger_index + 1;

                  if next_index > end_index {
                    next_index = home_index;
                  }
                } else {
                  next_index = trigger_index - 1;

                  if next_index < home_index {
                    next_index = end_index;
                  }
                }
              }
            }
            "ArrowDown" => {
              if orientation.get() == Orientation::Vertical {
                next_index = trigger_index + 1;

                if next_index > end_index {
                  next_index = home_index;
                }
              }
            }
            "ArrowLeft" => {
              if orientation.get() == Orientation::Horizontal {
                if is_direction_left_to_right.get() {
                  next_index = trigger_index - 1;

                  if next_index < home_index {
                    next_index = end_index;
                  }
                } else {
                  next_index = trigger_index + 1;

                  if next_index > end_index {
                    next_index = home_index;
                  }
                }
              }
            }
            "ArrowUp" => {
              if orientation.get() == Orientation::Vertical {
                next_index = trigger_index - 1;

                if next_index < home_index {
                  next_index = end_index;
                }
              }
            }
            _ => {}
          }

          let clamped_index = next_index % trigger_count;
          if let Some(trigger) = triggers.get(clamped_index) {
            _ = trigger.focus();
          }

          Some(())
        })();
      }
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}

#[derive(Clone)]
struct AccordionItemContextValue {
  open: Signal<bool>,
  disabled: Signal<bool>,
  trigger_id: Signal<String>,
}

#[component]
pub fn AccordionItem(
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(into)] value: MaybeSignal<String>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let state_context = use_context::<AccordionStateContextValue>()
    .expect("AccordionItem must be in an Accordion component");
  let value_context = use_context::<AccordionContextValue>()
    .expect("AccordionItem must be in an AccordionRoot component");

  let trigger_id = create_id();
  let is_open_value = value.clone();
  let is_open = Signal::derive(move || {
    value_context
      .value
      .get()
      .iter()
      .any(|item| (*item).eq(&is_open_value.get()) && !is_open_value.get().is_empty())
  });
  let is_disabled = Signal::derive(move || state_context.disabled.get() || disabled.get());

  provide_context(AccordionItemContextValue {
    open: Signal::derive(move || is_open.get()),
    disabled: Signal::derive(move || is_disabled.get()),
    trigger_id: Signal::derive(move || trigger_id.get()),
  });

  let open_value = value.clone();
  view! {
    <CollapsibleRoot
      {..attrs}
      attr:data-orientation=move || state_context.orientation.get().to_string()
      attr:data-state=move || if is_open.get() { "open" } else { "closed" }
      attr:data-disabled=disabled
      open=is_open
      disabled=is_disabled
      on_open_change=Callback::new(move |open| {
        if open {
          value_context.on_item_open.call(open_value.get());
        } else {
          value_context.on_item_close.call(open_value.get());
        }
      })
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </CollapsibleRoot>
  }
}

#[component]
pub fn AccordionHeader(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let AccordionStateContextValue { orientation, .. } =
    use_context().expect("AccordionHeader must be in an Accordion component");
  let AccordionItemContextValue { open, disabled, .. } =
    use_context().expect("AccordionHeader must be in an AccordionRoot component");

  view! {
    <Primitive
      {..attrs}
      attr:data-orientation=move || orientation.get().to_string()
      attr:data-state=move || if open.get() { "open" } else { "closed" }
      attr:data-disabled=disabled
      element=html::h3
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn AccordionTrigger(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let AccordionStateContextValue { orientation, .. } =
    use_context().expect("AccordionTrigger must be in an Accordion component");
  let AccordionItemContextValue {
    trigger_id, open, ..
  } = use_context().expect("AccordionTrigger must be in an AccordionRoot component");
  let AccordionCollapsibleContextValue { collapsible } =
    use_context().expect("AccordionTrigger must be in an AccordionRoot component");

  use_collection_item_ref(node_ref, AccordionCollectionItem);

  view! {
    <CollapsibleTrigger
      {..attrs}
      attr:data-orientation=move || orientation.get().to_string()
      attr:id=trigger_id
      attr:aria-disabled=move || open.get() && !collapsible.get()
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </CollapsibleTrigger>
  }
}

#[component]
pub fn AccordionContent(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let AccordionStateContextValue { orientation, .. } =
    use_context().expect("AccordionTrigger must be in an Accordion component");
  let AccordionItemContextValue { trigger_id, .. } =
    use_context().expect("AccordionTrigger must be in an AccordionRoot component");

  Effect::new(move |_| {
    let Some(node) = node_ref.get() else {
      return;
    };

    _ = node
      .style(
        "--primitive-accordion-content-width",
        "var(--primitive-collapsible-content-width)",
      )
      .style(
        "--primitive-accordion-content-height",
        "var(--primitive-collapsible-content-height)",
      );
  });

  view! {
    <CollapsibleContent
      {..attrs}
      attr:data-orientation=move || orientation.get().to_string()
      attr:aria-labelledby=trigger_id
      attr:role="region"
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </CollapsibleContent>
  }
}
