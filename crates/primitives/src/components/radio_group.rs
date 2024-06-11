use leptos::{
  ev::{keydown, keyup},
  html::AnyElement,
  *,
};
use leptos_use::{use_document, use_event_listener};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{FocusEvent, HtmlButtonElement, KeyboardEvent};

use crate::{
  components::{
    primitive::Primitive,
    radio::{Radio, RadioIndicator},
    roving_focus::{RovingFocusGroup, RovingFocusGroupItem},
  },
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    Direction, Orientation,
  },
  Attributes,
};

#[derive(Clone)]
struct RadioGroupContextValue {
  name: MaybeProp<String>,
  required: Signal<bool>,
  disabled: Signal<bool>,
  value: Signal<Option<String>>,
  on_value_change: Callback<String>,
}

#[component]
pub fn RadioGroupRoot(
  #[prop(optional, into)] name: MaybeProp<String>,
  #[prop(optional, into)] required: MaybeSignal<bool>,
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(optional, into)] should_loop: MaybeSignal<bool>,
  #[prop(optional, into)] value: MaybeProp<String>,
  #[prop(optional, into)] default_value: MaybeProp<String>,
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] direction: MaybeSignal<Direction>,

  #[prop(default=(|_|{}).into(), into)] on_value_change: Callback<String>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let (value, set_value) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || value.get()),
    default_value: Signal::derive(move || default_value.get()),
    on_change: on_value_change,
  });

  provide_context(RadioGroupContextValue {
    name,
    required: Signal::derive(move || required.get()),
    disabled: Signal::derive(move || disabled.get()),
    value: Signal::derive(move || value.get()),
    on_value_change: Callback::new(move |value| {
      set_value.set(value);
    }),
  });

  let children = StoredValue::new(children);

  view! {
    <RovingFocusGroup
      as_child=true
      orientation=Signal::derive(move || orientation.get())
      direction=Signal::derive(move || direction.get())
      should_loop=should_loop
    >
      <Primitive
        {..attrs}
        attr:role="radiogroup"
        attr:aria-required=required.clone()
        attr:aria-orientation=move || orientation.get().to_string()
        attr:data-disabled=disabled.clone()
        attr:dir=move || direction.get().to_string()
        element=html::div
        node_ref=node_ref
        as_child=as_child
      >
        {children.with_value(|children| children())}
      </Primitive>
    </RovingFocusGroup>
  }
}

#[component]
pub fn RadioGroupItem(
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(into)] value: MaybeSignal<String>,

  #[prop(default=(|_|{}).into(), into)] on_focus: Callback<FocusEvent>,
  #[prop(default=(|_|{}).into(), into)] on_key_down: Callback<KeyboardEvent>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let RadioGroupContextValue {
    disabled,
    value: context_value,
    required,
    name,
    on_value_change,
  } = use_context().expect("RadioGroupItem must be used in a RadioGroupRoot component");

  let is_disabled = Signal::derive(move || disabled.get() || disabled.get());

  let is_checked_value = value.clone();
  let is_checked = Signal::derive(move || context_value.get() == Some(is_checked_value.get()));
  let is_arrow_key_pressed = StoredValue::new(false);

  // _ = use_event_listener(use_document(), keydown, move |ev: KeyboardEvent| {
  //   if ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].contains(&ev.key().as_str()) {
  //     is_arrow_key_pressed.set_value(true);
  //   }
  // });

  // _ = use_event_listener(use_document(), keyup, move |ev: KeyboardEvent| {
  //   is_arrow_key_pressed.set_value(false);
  // });

  let children = StoredValue::new(children);
  let value = StoredValue::new(value);

  view! {
    <RovingFocusGroupItem
      as_child=true
      focusable=Signal::derive(move || !is_disabled.get())
      active=is_checked
    >
      <Radio
        value=value.get_value()
        disabled=is_disabled
        required=required
        checked=is_checked
        name=name.clone()
        on_check=Callback::new(move |_| on_value_change.call(value.get_value().get()))
        on:keydown=move |ev: KeyboardEvent| {
          on_key_down.call(ev.clone());

          if ev.key() == "Enter" {
            ev.prevent_default();
          }
        }
        on:focus=move |ev: FocusEvent| {
          on_focus.call(ev.clone());

          if is_arrow_key_pressed.get_value() {
            let Some(node) = node_ref.get() else {
              return;
            };

            let Some(node_el) = node.dyn_ref::<HtmlButtonElement>() else {
              return;
            };

            node_el.click();
          }
        }
        node_ref=node_ref
        attrs=attrs.clone()
        as_child=as_child
      >
        {children.with_value(|children| children())}
      </Radio>
    </RovingFocusGroupItem>
  }
}

#[component]
pub fn RadioGroupIndicator(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] children: Option<ChildrenFn>,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let children = StoredValue::new(children);

  view! {
    <RadioIndicator
      attrs=attrs
      node_ref=node_ref
      as_child=as_child
    >
      {children.with_value(|children| children.as_ref().map(|children| children()))}
    </RadioIndicator>
  }
}
