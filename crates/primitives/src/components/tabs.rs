use leptos::{html::AnyElement, *};
use web_sys::{FocusEvent, KeyboardEvent, MouseEvent};

use crate::{
  components::{
    presence::create_presence,
    primitive::Primitive,
    roving_focus::{RovingFocusGroup, RovingFocusGroupItem},
  },
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    create_id::create_id,
    Direction, Orientation,
  },
  Attributes,
};

#[derive(Clone)]
struct TabsContextValue {
  base_id: Signal<String>,
  value: Signal<Option<String>>,
  on_value_change: Callback<String>,
  orientation: Signal<Orientation>,
  direction: Signal<Direction>,
  activation_mode: Signal<ActivationMode>,
}

#[derive(Clone, Default, PartialEq)]
pub enum ActivationMode {
  #[default]
  Automatic,
  Manual,
}

#[component]
pub fn TabsRoot(
  #[prop(optional)] value: Option<MaybeSignal<String>>,
  #[prop(optional)] default_value: Option<MaybeSignal<String>>,
  #[prop(optional)] on_value_change: Option<Callback<String>>,
  #[prop(optional)] orientation: Option<MaybeSignal<Orientation>>,
  #[prop(optional)] direction: Option<MaybeSignal<Direction>>,
  #[prop(optional)] activation_mode: Option<MaybeSignal<ActivationMode>>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: Children,
) -> impl IntoView {
  let (value, set_value) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || value.as_ref().map(|value| value.get())),
    default_value: Signal::derive(move || {
      default_value
        .as_ref()
        .map(|default_value| default_value.get())
    }),
    on_change: Callback::new(move |value| {
      if let Some(on_value_change) = on_value_change {
        on_value_change.call(value);
      }
    }),
  });

  provide_context(TabsContextValue {
    base_id: create_id(),
    value: Signal::derive(move || value.get()),
    on_value_change: Callback::new(move |value| {
      set_value.set(value);
    }),
    direction: Signal::derive(move || {
      direction
        .map(|direction| direction.get())
        .unwrap_or_default()
    }),
    orientation: Signal::derive(move || {
      orientation
        .map(|orientation| orientation.get())
        .unwrap_or_default()
    }),
    activation_mode: Signal::derive(move || {
      activation_mode
        .as_ref()
        .map(|activation_mode| activation_mode.get())
        .unwrap_or_default()
    }),
  });

  view! {
    <Primitive
      element=html::div
      attrs=attrs
      node_ref=node_ref
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn TabsList(
  #[prop(optional)] should_loop: Option<MaybeSignal<bool>>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: Children,
) -> impl IntoView {
  let context =
    use_context::<TabsContextValue>().expect("TabsList must be used in a TabsRoot component");

  let mut merged_attrs = attrs.clone();
  merged_attrs.extend([
    ("role", "tablist".into_attribute()),
    (
      "aria-orientation",
      (move || context.orientation.get().to_string()).into_attribute(),
    ),
  ]);

  view! {
    <RovingFocusGroup
      as_child=true
      orientation=Signal::derive(move || context.orientation.get()).into()
      direction=Signal::derive(move || context.direction.get()).into()
      should_loop=Signal::derive(move || should_loop.map(|should_loop| should_loop.get()).unwrap_or(true)).into()
    >
      <Primitive
        element=html::div
        attrs=merged_attrs
        node_ref=node_ref
      >
        {children()}
      </Primitive>
    </RovingFocusGroup>
  }
}

#[component]
pub fn TabsTrigger(
  #[prop(optional)] value: MaybeSignal<String>,
  #[prop(optional)] disabled: Option<MaybeSignal<bool>>,
  #[prop(optional)] on_mouse_down: Option<Callback<MouseEvent>>,
  #[prop(optional)] on_key_down: Option<Callback<KeyboardEvent>>,
  #[prop(optional)] on_focus: Option<Callback<FocusEvent>>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: Children,
) -> impl IntoView {
  let context =
    use_context::<TabsContextValue>().expect("TabsTrigger must be used in a TabsRoot component");

  let trigger_value = value.clone();
  let trigger_id =
    Signal::derive(move || format!("{}-trigger-{}", context.base_id.get(), trigger_value.get()));

  let content_value = value.clone();
  let content_id =
    Signal::derive(move || format!("{}-content-{}", context.base_id.get(), content_value.get()));

  let is_selected_value = value.clone();
  let is_selected = Signal::derive(move || context.value.get() == Some(is_selected_value.get()));

  let data_disabled = disabled;
  let mut merged_attrs = attrs.clone();
  merged_attrs.extend([
    ("type", "button".into_attribute()),
    ("role", "tab".into_attribute()),
    (
      "aria-selected",
      Signal::derive(move || is_selected.get()).into_attribute(),
    ),
    (
      "aria-controls",
      Signal::derive(move || content_id.get()).into_attribute(),
    ),
    (
      "data-state",
      Signal::derive(move || {
        if is_selected.get() {
          "active"
        } else {
          "inactive"
        }
      })
      .into_attribute(),
    ),
    (
      "data-disabled",
      Signal::derive(move || data_disabled.map(|disabled| disabled.get().then_some("")))
        .into_attribute(),
    ),
    (
      "disabled",
      Signal::derive(move || disabled.map(|disabled| disabled.get())).into_attribute(),
    ),
    (
      "id",
      Signal::derive(move || trigger_id.get()).into_attribute(),
    ),
  ]);

  let keydown_value = value.clone();
  let focus_value = value.clone();
  view! {
    <RovingFocusGroupItem
      as_child=true
      focusable=Signal::derive(move || !disabled.map(|disabled| disabled.get()).unwrap_or(false)).into()
      active=Signal::derive(move || is_selected.get()).into()
    >
      <Primitive
        element=html::button
        attrs=merged_attrs
        node_ref=node_ref
        on:mousedown=move|ev: MouseEvent| {
          if let Some(on_mouse_down) = on_mouse_down {
            on_mouse_down.call(ev.clone());
          }

          if !disabled.map(|disabled| disabled.get()).unwrap_or(false) && ev.button() == 0 && !ev.ctrl_key() {
            context.on_value_change.call(value.get());
          } else {
            ev.prevent_default();
          }
        }
        on:keydown=move |ev: KeyboardEvent| {
          if let Some(on_key_down) = on_key_down {
            on_key_down.call(ev.clone());
          }

          if [" ", "Enter"].contains(&ev.key().as_str()) {
            context.on_value_change.call(keydown_value.get());
          }
        }
        on:focus=move |ev: FocusEvent| {
          if let Some(on_focus) = on_focus {
            on_focus.call(ev.clone());
          }

          let is_automatic_activation = context.activation_mode.get() != ActivationMode::Manual;

          if !is_selected.get() && !disabled.map(|disabled| disabled.get()).unwrap_or(false) && is_automatic_activation {
            context.on_value_change.call(focus_value.get());
          }
        }
      >
        {children()}
      </Primitive>
    </RovingFocusGroupItem>
  }
}

#[component]
pub fn TabsContent(
  #[prop(optional)] value: MaybeSignal<String>,
  #[prop(optional)] force_mount: Option<MaybeSignal<bool>>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: ChildrenFn,
) -> impl IntoView {
  let context =
    use_context::<TabsContextValue>().expect("TabsContent must be used in a TabsRoot component");

  let trigger_value = value.clone();
  let trigger_id =
    Signal::derive(move || format!("{}-trigger-{}", context.base_id.get(), trigger_value.get()));
  let content_value = value.clone();
  let content_id =
    Signal::derive(move || format!("{}-content-{}", context.base_id.get(), content_value.get()));

  let is_selected_value = value.clone();
  let is_selected = Signal::derive(move || context.value.get() == Some(is_selected_value.get()));
  let is_mount_animation_prevented = StoredValue::new(is_selected.get_untracked());

  let is_present = Signal::derive(move || {
    is_selected.get()
      || force_mount
        .map(|force_mount| force_mount.get())
        .unwrap_or(false)
  });

  let presence = create_presence(is_present, node_ref);

  Effect::new(move |_| {
    let Ok(animation_frame_handle) = request_animation_frame_with_handle(move || {
      is_mount_animation_prevented.set_value(false);
    }) else {
      return;
    };

    on_cleanup(move || {
      animation_frame_handle.cancel();
    });
  });

  Effect::new(move |_| {
    let Some(node) = node_ref.get() else {
      return;
    };

    _ = presence.get();

    if is_mount_animation_prevented.get_value() {
      _ = node.style("animation-duration", "0s");
    }
  });

  let mut merged_attrs = vec![
    ("role", "tabpanel".into_attribute()),
    (
      "data-state",
      (move || {
        if is_selected.get() {
          "active"
        } else {
          "inactive"
        }
      })
      .into_attribute(),
    ),
    (
      "data-orientation",
      (move || context.orientation.get().to_string()).into_attribute(),
    ),
    (
      "aria-labelledby",
      (move || trigger_id.get()).into_attribute(),
    ),
    ("hidden", (move || !is_present.get()).into_attribute()),
    ("id", (move || content_id.get()).into_attribute()),
    ("tabindex", 0.into_attribute()),
  ];

  merged_attrs.extend(attrs.clone());

  let children = StoredValue::new(children);

  view! {
      <Show when=move || presence.get()>
        <Primitive
            element=html::div
            attrs=merged_attrs.clone()
            node_ref=node_ref
        >
            {children.with_value(|children| children())}
        </Primitive>
      </Show>
  }
}
