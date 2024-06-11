use leptos::{
  html::{AnyElement, Input},
  *,
};
use leptos_use::{use_element_size, UseElementSizeReturn};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
  js_sys::{Array, Function, JsString, Object, Reflect},
  Event, EventInit, MouseEvent,
};

use crate::{
  components::primitive::Primitive,
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    create_previous::create_previous,
  },
  Attributes,
};

#[derive(Clone)]
struct SwitchContextValue {
  checked: Signal<bool>,
  disabled: Signal<bool>,
}

#[component]
pub fn SwitchRoot(
  #[prop(optional, into)] checked: MaybeProp<bool>,
  #[prop(optional, into)] default_checked: MaybeProp<bool>,
  #[prop(optional, into)] value: MaybeProp<String>,
  #[prop(optional, into)] name: MaybeProp<String>,
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(optional, into)] required: MaybeSignal<bool>,

  #[prop(default=(|_|{}).into(), into)] on_checked_change: Callback<bool>,
  #[prop(default=(|_|{}).into(), into)] on_click: Callback<MouseEvent>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let node_ref = NodeRef::<AnyElement>::new();
  let (is_form_control, set_is_form_control) = create_signal(true);

  let has_consumer_stopped_propagation = StoredValue::new(false);

  let (checked, set_checked) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || checked.get()),
    default_value: Signal::derive(move || default_checked.get()),
    on_change: on_checked_change,
  });

  Effect::new(move |_| {
    set_is_form_control.set(if let Some(foo) = node_ref.get() {
      foo.closest("form").ok().flatten().is_some()
    } else {
      true
    });
  });

  provide_context(SwitchContextValue {
    checked: Signal::derive(move || checked.get().unwrap_or(false)),
    disabled: Signal::derive(move || disabled.get()),
  });

  let attr_value = value.clone();

  view! {
    <Primitive
      {..attrs}
      attr:type="button"
      attr:role="switch"
      attr:aria-checked=checked
      attr:aria-required=required
      attr:data-state=move || {
        if checked.get().unwrap_or(false) {
          "checked"
        } else {
          "unchecked"
        }
      }
      attr:data-disabled=disabled
      attr:value=attr_value
      element=html::button
      on:click=move |ev: MouseEvent| {
        on_click.call(ev.clone());

        set_checked.update(|checked| *checked = Some(!checked.unwrap_or(false)));

        if is_form_control.get() {
          // has_consumer_stopped_propagation.set_value(ev.is_propagation_stopped());

          if !has_consumer_stopped_propagation.get_value() {
            ev.stop_propagation();
          }
        }
      }
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>

    <Show when=move || is_form_control.get()>
      <BubbleInput
        checked=Signal::derive(move || checked.get().unwrap_or(false))
        bubbles=Signal::derive(move || !has_consumer_stopped_propagation.get_value())
        name=name.clone()
        value=value.clone()
        disabled=Signal::derive(move || disabled.get())
        required=Signal::derive(move || required.get())
        control=node_ref
      />
    </Show>
  }
}

#[component]
pub fn SwitchThumb(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] children: Option<ChildrenFn>,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let SwitchContextValue { checked, disabled } =
    use_context().expect("SwitchThumb must be used in a SwitchRoot component");

  let children = StoredValue::new(children);

  view! {
    <Primitive
      {..attrs}
      attr:data-state=move || {
        if checked.get() {
          "checked"
        } else {
          "unchecked"
        }
      }
      attr:data-disabled=move || disabled.get().then_some("")
      element=html::span
      node_ref=node_ref
      as_child=as_child
    >
      {children.with_value(|children| children.as_ref().map(|children| children()))}
    </Primitive>
  }
}

#[component]
pub fn BubbleInput(
  checked: Signal<bool>,
  bubbles: Signal<bool>,
  #[prop(into)] name: MaybeProp<String>,
  #[prop(into)] value: MaybeProp<String>,
  disabled: Signal<bool>,
  required: Signal<bool>,
  control: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
) -> impl IntoView {
  let node_ref = NodeRef::<Input>::new();
  let prev_checked = create_previous(Signal::derive(move || checked.get()));
  let UseElementSizeReturn { width, height } = use_element_size(control);

  Effect::new(move |_| {
    (|| {
      let input = node_ref.get()?;
      let input_el = window().get("HTMLInputElement")?;
      let input_proto = Reflect::get(&input_el, &JsString::from("prototype"))
        .ok()?
        .dyn_into::<Object>()
        .ok()?;

      let input_descriptor_set = Reflect::get(
        &Object::get_own_property_descriptor(&input_proto, &JsString::from("checked")),
        &JsString::from("set"),
      )
      .ok()?
      .dyn_into::<Function>()
      .ok()?;

      if prev_checked.get() != checked.get() {
        let mut ev_options = EventInit::new();
        ev_options.bubbles(bubbles.get());

        let ev = Event::new_with_event_init_dict("click", &ev_options).ok()?;

        _ = Reflect::apply(
          &input_descriptor_set,
          &input,
          &Array::from_iter([JsValue::from_bool(checked.get())]),
        );

        _ = input.dispatch_event(&ev);
      }

      Some(())
    })();
  });

  view! {
    <input
      type="checkbox"
      aria-hidden
      checked=Signal::derive(move || checked.get()).into_attribute()
      tabindex=(-1).into_attribute()
      node_ref=node_ref
      name=name.into_attribute()
      value=Signal::derive(move || value.get().unwrap_or("on".into())).into_attribute()
      disabled=Signal::derive(move || disabled.get()).into_attribute()
      required=Signal::derive(move || required.get()).into_attribute()
      style:position="absolute"
      style:pointer-events="none"
      style:opacity="0"
      style:margin="0"
      style:width=move || width.get()
      style:height=move || height.get()
      {..attrs}
    />
  }
}
