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
  components::{presence::create_presence, primitive::Primitive},
  util::create_previous::create_previous,
  Attributes,
};

#[derive(Clone)]
struct RadioContextValue {
  checked: Signal<bool>,
  disabled: Signal<bool>,
}

#[component]
pub fn Radio(
  #[prop(optional, into)] value: MaybeSignal<String>,
  #[prop(optional, into)] checked: MaybeSignal<bool>,
  #[prop(optional, into)] required: MaybeSignal<bool>,
  #[prop(default=(|_|{}).into(), into)] on_check: Callback<()>,
  #[prop(default=(|_|{}).into(), into)] on_click: Callback<MouseEvent>,

  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(optional, into)] name: MaybeProp<String>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let (is_form_control, set_is_form_control) = create_signal(true);
  let has_consumer_stopped_propagation = StoredValue::new(false);

  Effect::new(move |_| {
    set_is_form_control.set(if let Some(form) = node_ref.get() {
      form.closest("form").ok().flatten().is_some()
    } else {
      true
    });
  });

  provide_context(RadioContextValue {
    checked: Signal::derive(move || checked.get()),
    disabled: Signal::derive(move || disabled.get()),
  });

  view! {
    <Primitive
      {..attrs}
      attr:type="button"
      attr:role="radio"
      attr:aria-checked=move || checked.get().to_string()
      attr:data-state=move || {
        if checked.get() {
          "checked"
        } else {
          "unchecked"
        }
      }
      attr:data-disabled=disabled
      attr:disabled=move || disabled.get().then_some("")
      attr:value=value.clone()
      element=html::button
      on:click=move |ev: MouseEvent| {
        on_click.call(ev.clone());

        if !checked.get() {
          on_check.call(())
        }

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
        checked=Signal::derive(move || checked.get())
        bubbles=Signal::derive(move || !has_consumer_stopped_propagation.get_value())
        name=name.clone()
        value=value.clone()
        required=Signal::derive(move || required.get())
        disabled=Signal::derive(move || disabled.get())
        control=node_ref
      />
    </Show>
  }
}

#[component]
pub fn RadioIndicator(
  #[prop(optional, into)] force_mount: MaybeSignal<bool>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let RadioContextValue { checked, disabled } =
    use_context().expect("RadioIndicator must be used in a Radio component");

  let is_present = Signal::derive(move || force_mount.get() || checked.get());
  let presence = create_presence(is_present, node_ref);

  let children = StoredValue::new(children);

  view! {
    <Show when=move || presence.get()>
        <Primitive
          {..attrs.clone()}
          attr:data-state=move || {
            if checked.get() {
              "checked"
            } else {
              "unchecked"
            }
          }
          attr:data-disabled=disabled.clone()
          element=html::span
          node_ref=node_ref
          as_child=as_child
        >
          {children.with_value(|children| children())}
        </Primitive>
    </Show>
  }
}

#[component]
fn BubbleInput(
  checked: Signal<bool>,
  bubbles: Signal<bool>,
  #[prop(into)] name: MaybeProp<String>,
  value: MaybeSignal<String>,
  required: Signal<bool>,
  disabled: Signal<bool>,
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
      name=name.into_attribute()
      value=value.into_attribute()
      required=required.into_attribute()
      disabled=disabled.into_attribute()
      checked=checked.into_attribute()
      tabindex=(-1).into_attribute()
      node_ref=node_ref
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
