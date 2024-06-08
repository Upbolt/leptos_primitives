use leptos::{html::AnyElement, leptos_dom::helpers::AnimationFrameRequestHandle, *};
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
  #[prop(optional, into)] value: Option<MaybeSignal<String>>,
  #[prop(optional, into)] default_value: Option<MaybeSignal<String>>,
  #[prop(default=Callback::new(|_:String|{}), into)] on_value_change: Callback<String>,
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] direction: MaybeSignal<Direction>,
  #[prop(optional, into)] activation_mode: MaybeSignal<ActivationMode>,

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
      on_value_change.call(value);
    }),
  });

  provide_context(TabsContextValue {
    base_id: create_id(),
    value: Signal::derive(move || value.get()),
    on_value_change: Callback::new(move |value| {
      set_value.set(value);
    }),
    direction: Signal::derive(move || direction.get()),
    orientation: Signal::derive(move || orientation.get()),
    activation_mode: Signal::derive(move || activation_mode.get()),
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
  #[prop(default=true.into(), into)] should_loop: MaybeSignal<bool>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: Children,
) -> impl IntoView {
  let TabsContextValue {
    orientation,
    direction,
    ..
  } = use_context().expect("TabsList must be used in a TabsRoot component");

  let mut merged_attrs = attrs.clone();
  merged_attrs.extend([
    ("role", "tablist".into_attribute()),
    (
      "aria-orientation",
      (move || orientation.get().to_string()).into_attribute(),
    ),
  ]);

  view! {
    <RovingFocusGroup
      as_child=true
      orientation=Some(orientation.into())
      direction=Some(direction.into())
      should_loop=Signal::derive(move || should_loop.get())
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
  #[prop(optional, into)] value: MaybeSignal<String>,
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(default=Callback::new(|_:MouseEvent|{}), into)] on_mouse_down: Callback<MouseEvent>,
  #[prop(default=Callback::new(|_:KeyboardEvent|{}), into)] on_key_down: Callback<KeyboardEvent>,
  #[prop(default=Callback::new(|_:FocusEvent|{}), into)] on_focus: Callback<FocusEvent>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: Children,
) -> impl IntoView {
  let TabsContextValue {
    base_id,
    value: context_value,
    on_value_change,
    activation_mode,
    ..
  } = use_context().expect("TabsTrigger must be used in a TabsRoot component");

  let trigger_value = value.clone();
  let trigger_id =
    Signal::derive(move || format!("{}-trigger-{}", base_id.get(), trigger_value.get()));

  let content_value = value.clone();
  let content_id =
    Signal::derive(move || format!("{}-content-{}", base_id.get(), content_value.get()));

  let is_selected_value = value.clone();
  let is_selected = Signal::derive(move || context_value.get() == Some(is_selected_value.get()));

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
      Signal::derive(move || data_disabled.get().then_some("")).into_attribute(),
    ),
    ("disabled", disabled.into_attribute()),
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
      focusable=Signal::derive(move || !disabled.get())
      active=is_selected
    >
      <Primitive
        element=html::button
        attrs=merged_attrs
        node_ref=node_ref
        on:mousedown=move|ev: MouseEvent| {
            on_mouse_down.call(ev.clone());

          if !disabled.get() && ev.button() == 0 && !ev.ctrl_key() {
            on_value_change.call(value.get());
          } else {
            ev.prevent_default();
          }
        }
        on:keydown=move |ev: KeyboardEvent| {
            on_key_down.call(ev.clone());

          if [" ", "Enter"].contains(&ev.key().as_str()) {
            on_value_change.call(keydown_value.get());
          }
        }
        on:focus=move |ev: FocusEvent| {
            on_focus.call(ev.clone());

          let is_automatic_activation = activation_mode.get() != ActivationMode::Manual;

          if !is_selected.get() && !disabled.get() && is_automatic_activation {
            on_value_change.call(focus_value.get());
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
  #[prop(optional, into)] value: MaybeSignal<String>,
  #[prop(optional, into)] force_mount: MaybeSignal<bool>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: ChildrenFn,
) -> impl IntoView {
  let TabsContextValue {
    base_id,
    value: context_value,
    orientation,
    ..
  } = use_context().expect("TabsContent must be used in a TabsRoot component");

  let trigger_value = value.clone();
  let trigger_id =
    Signal::derive(move || format!("{}-trigger-{}", base_id.get(), trigger_value.get()));
  let content_value = value.clone();
  let content_id =
    Signal::derive(move || format!("{}-content-{}", base_id.get(), content_value.get()));

  let is_selected_value = value.clone();
  let is_selected = Signal::derive(move || value.get() == is_selected_value.get());
  let is_mount_animation_prevented = StoredValue::new(is_selected.get_untracked());

  let is_present = Signal::derive(move || is_selected.get() || force_mount.get());

  let presence = create_presence(is_present, node_ref);
  let animation_frame_handle = StoredValue::<Option<AnimationFrameRequestHandle>>::new(None);

  Effect::new(move |_| {
    if let Ok(handle) = request_animation_frame_with_handle(move || {
      is_mount_animation_prevented.set_value(false);
    }) {
      animation_frame_handle.set_value(Some(handle));
    }
  });

  on_cleanup(move || {
    if let Some(handle) = animation_frame_handle.get_value() {
      handle.cancel();
    }
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
      (move || orientation.get().to_string()).into_attribute(),
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
