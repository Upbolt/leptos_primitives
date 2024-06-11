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
  #[prop(optional, into)] value: MaybeProp<String>,
  #[prop(optional, into)] default_value: MaybeProp<String>,
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] direction: MaybeSignal<Direction>,
  #[prop(optional, into)] activation_mode: MaybeSignal<ActivationMode>,

  #[prop(default=(|_|{}).into(), into)] on_value_change: Callback<String>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let (value, set_value) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || value.get()),
    default_value: Signal::derive(move || default_value.get()),
    on_change: on_value_change,
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
      node_ref=node_ref
      attrs=attrs
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn TabsList(
  #[prop(default=true.into(), into)] should_loop: MaybeSignal<bool>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let TabsContextValue {
    orientation,
    direction,
    ..
  } = use_context().expect("TabsList must be used in a TabsRoot component");

  let children = StoredValue::new(children);

  view! {
    <RovingFocusGroup
      as_child=true
      orientation=Signal::derive(move || orientation.get())
      direction=Signal::derive(move || direction.get())
      should_loop=Signal::derive(move || should_loop.get())
    >
      <Primitive
        {..attrs}
        attr:role="tablist"
        attr:aria-orientation=move || orientation.get().to_string()
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
pub fn TabsTrigger(
  #[prop(optional, into)] value: MaybeSignal<String>,
  #[prop(optional, into)] disabled: MaybeSignal<bool>,

  #[prop(default=(|_|{}).into(), into)] on_mouse_down: Callback<MouseEvent>,
  #[prop(default=(|_|{}).into(), into)] on_key_down: Callback<KeyboardEvent>,
  #[prop(default=(|_|{}).into(), into)] on_focus: Callback<FocusEvent>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
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

  let children = StoredValue::new(children);
  let value = StoredValue::new(value);

  view! {
    <RovingFocusGroupItem
      as_child=true
      focusable=Signal::derive(move || !disabled.get())
      active=is_selected
    >
      <Primitive
        {..attrs}
        attr:type="button"
        attr:role="tab"
        attr:aria-selected=is_selected
        attr:aria-controls=content_id
        attr:data-state=move || {
          if is_selected.get() {
            "active"
          } else {
            "inactive"
          }
        }
        attr:data-disabled=move || disabled.get().then_some("")
        attr:disabled=disabled
        attr:id=trigger_id
        element=html::button
        on:mousedown=move|ev: MouseEvent| {
          on_mouse_down.call(ev.clone());

          if !disabled.get() && ev.button() == 0 && !ev.ctrl_key() {
            on_value_change.call(value.get_value().get());
          } else {
            ev.prevent_default();
          }
        }
        on:keydown=move |ev: KeyboardEvent| {
          on_key_down.call(ev.clone());

          if [" ", "Enter"].contains(&ev.key().as_str()) {
            on_value_change.call(value.get_value().get());
          }
        }
        on:focus=move |ev: FocusEvent| {
          on_focus.call(ev.clone());

          let is_automatic_activation = activation_mode.get() != ActivationMode::Manual;

          if !is_selected.get() && !disabled.get() && is_automatic_activation {
            on_value_change.call(value.get_value().get());
          }
        }
        node_ref=node_ref
        as_child=as_child
      >
        {children.with_value(|children| children())}
      </Primitive>
    </RovingFocusGroupItem>
  }
}

#[component]
pub fn TabsContent(
  #[prop(optional, into)] value: MaybeSignal<String>,
  #[prop(optional, into)] force_mount: MaybeSignal<bool>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
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
  let is_selected = Signal::derive(move || context_value.get() == Some(is_selected_value.get()));
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

  let children = StoredValue::new(children);

  view! {
    <Show when=move || presence.get()>
      <Primitive
        {..attrs.clone()}
        attr:role="tabpanel"
        attr:data-state=move || {
          if is_selected.get() {
            "active"
          } else {
            "inactive"
          }
        }
        attr:data-orientation=move || orientation.get().to_string()
        attr:aria-labelledby=trigger_id.clone()
        attr:hidden=move || !is_present.get()
        attr:id=content_id.clone()
        attr:tabindex=0
        element=html::div
        node_ref=node_ref
        as_child=as_child
      >
        {children.with_value(|children| children())}
      </Primitive>
    </Show>
  }
}
