use leptos::{
  html::{AnyElement, Input},
  *,
};

use leptos_use::{use_element_size, UseElementSizeReturn};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
  js_sys::{Array, Function, JsString, Object, Reflect},
  Event, EventInit, HtmlButtonElement, KeyboardEvent, MouseEvent,
};

use crate::{
  components::{presence::create_presence, primitive::Primitive},
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    create_previous::create_previous,
  },
  Attributes,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CheckedState {
  Checked(bool),
  Indeterminate,
}

#[derive(Clone)]
struct CheckboxValueContext {
  state: Signal<CheckedState>,
  disabled: Signal<bool>,
}

#[component]
pub fn CheckboxRoot(
  #[prop(optional, into)] required: MaybeSignal<bool>,
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(optional, into)] checked: MaybeProp<CheckedState>,
  #[prop(optional, into)] default_checked: MaybeProp<CheckedState>,

  #[prop(default=(|_|{}).into(), into)] on_checked_change: Callback<CheckedState>,
  #[prop(default=(|_|{}).into(), into)] on_click: Callback<MouseEvent>,
  #[prop(default=(|_|{}).into(), into)] on_key_down: Callback<KeyboardEvent>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let has_consumer_stropped_propagation = StoredValue::new(false);

  let (is_form_control, set_is_form_control) = create_signal(true);

  let (checked, set_checked) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || checked.get()),
    default_value: Signal::derive(move || default_checked.get()),
    on_change: Callback::new(move |value| {
      on_checked_change.call(value);
    }),
  });

  let initial_checked_state = StoredValue::new(checked.get_untracked());

  Effect::new(move |_| {
    let Some(node) = node_ref.get() else {
      return;
    };

    let Some(button) = node.dyn_ref::<HtmlButtonElement>() else {
      return;
    };

    let Some(form) = button.form() else {
      return;
    };

    let reset_set_checked = set_checked;
    let reset = Closure::<dyn FnMut()>::new(move || {
      reset_set_checked.set(
        initial_checked_state
          .get_value()
          .unwrap_or(CheckedState::Checked(false)),
      );
    });

    _ = form.add_event_listener_with_callback("reset", reset.as_ref().unchecked_ref());

    on_cleanup(move || {
      _ = form.remove_event_listener_with_callback("reset", reset.as_ref().unchecked_ref());

      reset.forget();
    });
  });

  Effect::new(move |_| {
    set_is_form_control.set(if let Some(foo) = node_ref.get() {
      foo.closest("form").ok().flatten().is_some()
    } else {
      true
    });
  });

  provide_context(CheckboxValueContext {
    state: Signal::derive(move || checked.get().unwrap_or(CheckedState::Checked(false))),
    disabled: Signal::derive(move || disabled.get()),
  });

  let bubble_ref = NodeRef::<Input>::new();

  Effect::new(move |_| {
    let Some(node) = bubble_ref.get() else {
      return;
    };

    _ = node.style("transform", "translateX(-100%)");
  });

  view! {
    <Primitive
      {..attrs}
      element=html::button
      attr:type="button"
      attr:role="checkbox"
      attr:aria-checked=move || {
        checked.get().map(|checked| match checked {
          CheckedState::Checked(checked) => checked.into_attribute(),
          CheckedState::Indeterminate => "mixed".into_attribute(),
        })
      }
      attr:aria-required=required
      attr:data-state=move || {
        checked.get().map(|checked| match checked {
          CheckedState::Checked(checked) => {
            if checked {
              "checked"
            } else {
              "unchecked"
            }
          }
          CheckedState::Indeterminate => "indeterminate",
        })
      }
      attr:data-disabled=disabled
      attr:disabled=disabled
      on:keydown=move |ev: KeyboardEvent| {
        on_key_down.call(ev.clone());

        if ev.key() == "Enter" {
          ev.prevent_default();
        }
      }
      on:click=move |ev: MouseEvent| {
        on_click.call(ev.clone());

        set_checked.update(|checked| {
          *checked = Some(match checked.as_ref().unwrap_or(&CheckedState::Checked(false)) {
            CheckedState::Checked(checked) => CheckedState::Checked(!checked),
            CheckedState::Indeterminate => CheckedState::Checked(true),
          });
        });

        if is_form_control.get() {
          // if !ev.is_propagation_stopped()
          ev.stop_propagation();
        }
      }
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
    <Show when=move || is_form_control.get()>
      <BubbleInput
        checked=Signal::derive(move || checked.get().unwrap_or(CheckedState::Checked(false)))
        bubbles=Signal::derive(move || false)
        control=node_ref
        node_ref=bubble_ref
      />
    </Show>
  }
}

#[component]
pub fn CheckboxIndicator(
  #[prop(optional, into)] force_mount: MaybeSignal<bool>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let CheckboxValueContext { state, disabled } = use_context::<CheckboxValueContext>()
    .expect("CheckboxIndicator must be used inside of a CheckboxRoot component");

  let is_present =
    Signal::derive(move || force_mount.get() || state.get() != CheckedState::Checked(false));

  let presence = create_presence(is_present, node_ref);

  let children = StoredValue::new(children);

  view! {
    <Show when=move || presence.get()>
      <Primitive
        {..attrs.clone()}
        attr:data-state=move || match state.get() {
          CheckedState::Checked(checked) => {
            if checked {
              "checked"
            } else {
              "unchecked"
            }
          }
          CheckedState::Indeterminate => "indeterminate",
        }
        attr:disabled=disabled.clone()
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
  checked: Signal<CheckedState>,
  control: NodeRef<AnyElement>,
  bubbles: Signal<bool>,

  node_ref: NodeRef<Input>,
  #[prop(attrs)] attrs: Attributes,
) -> impl IntoView {
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

        _ = Reflect::set(
          &input,
          &JsString::from("indeterminate"),
          &JsValue::from_bool(checked.get() == CheckedState::Indeterminate),
        );

        _ = Reflect::apply(
          &input_descriptor_set,
          &input,
          &Array::from_iter([JsValue::from_bool(match checked.get() {
            CheckedState::Indeterminate => false,
            CheckedState::Checked(checked) => checked,
          })]),
        );

        _ = input.dispatch_event(&ev);
      }

      Some(())
    })();
  });

  view! {
    <input
      {..attrs}
      type="checkbox"
      aria-hidden="true"
      checked=(move || match checked.get() { CheckedState::Checked(checked) => checked, CheckedState::Indeterminate => false }).into_attribute()
      tabindex=(-1).into_attribute()
      node_ref=node_ref
      style:position="absolute"
      style:pointer-events="none"
      style:opacity="0"
      style:margin="0"
      style:width=move || format!("{}px", width.get())
      style:height=move || format!("{}px", height.get())
    />
  }
}
