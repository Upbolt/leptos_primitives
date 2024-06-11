use leptos::{html::AnyElement, leptos_dom::helpers::AnimationFrameRequestHandle, *};
use wasm_bindgen::JsValue;
use web_sys::{js_sys::Object, CssStyleDeclaration, MouseEvent};

use crate::{
  components::primitive::Primitive,
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    create_id::create_id,
  },
  Attributes,
};

use super::presence::create_presence;

#[derive(Clone)]
struct CollapsibleContextValue {
  content_id: Signal<String>,
  disabled: Signal<bool>,
  open: Signal<bool>,
  on_open_toggle: Callback<()>,
}

#[component]
pub fn CollapsibleRoot(
  #[prop(optional, into)] open: MaybeSignal<bool>,
  #[prop(optional, into)] default_open: MaybeSignal<bool>,
  #[prop(optional, into)] disabled: MaybeSignal<bool>,

  #[prop(default=(|_|{}).into(), into)] on_open_change: Callback<bool>,
  #[prop(default=(|_|{}).into(), into)] on_click: Callback<MouseEvent>,

  #[prop(optional, into)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let (open, set_open) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || Some(open.get())),
    default_value: Signal::derive(move || Some(default_open.get())),
    on_change: Callback::new(move |value| {
      on_open_change.call(value);
    }),
  });

  provide_context(CollapsibleContextValue {
    open: Signal::derive(move || open.get().unwrap_or(false)),
    disabled: Signal::derive(move || disabled.get()),
    content_id: create_id(),
    on_open_toggle: Callback::new(move |_| {
      set_open.update(|open| *open = Some(!open.unwrap_or(false)))
    }),
  });

  view! {
    <Primitive
      {..attrs}
      attr:data-state=move || {
        if open.get().unwrap_or(false) {
          "open"
        } else {
          "closed"
        }
      }
      attr:data-disabled=disabled
      element=html::div
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn CollapsibleTrigger(
  #[prop(default=(|_|{}).into(), into)] on_click: Callback<MouseEvent>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let CollapsibleContextValue {
    content_id,
    disabled,
    open,
    on_open_toggle,
  } = use_context::<CollapsibleContextValue>()
    .expect("CollapsibleTrigger must be used in a CollapsibleRoot component");

  view! {
    <Primitive
      {..attrs}
      attr:aria-controls=content_id
      attr:aria-expanded=open
      attr:data-state=move || if open.get() { "open" } else { "closed" }
      attr:data-disabled=disabled
      attr:disabled=disabled
      element=html::button
      on:click=move |ev: MouseEvent| {
        on_click.call(ev);
        on_open_toggle.call(());
      }
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn CollapsibleContent(
  #[prop(optional, into)] force_mount: MaybeSignal<bool>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let CollapsibleContextValue { open, .. } = use_context::<CollapsibleContextValue>()
    .expect("CollapsibleContent must be used in a CollapsibleRoot component");

  let is_present = Signal::derive(move || open.get() || force_mount.get());

  let presence = create_presence(is_present, node_ref);
  let children = StoredValue::new(children);

  view! {
    <Show when=move || presence.get()>
      <CollapsibleContentImpl
        is_present=presence
        node_ref=node_ref
        attrs=attrs.clone()
        as_child=as_child
      >
        {children.with_value(|children| children())}
      </CollapsibleContentImpl>
    </Show>
  }
}

#[component]
fn CollapsibleContentImpl(
  is_present: Signal<bool>,

  node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let CollapsibleContextValue {
    content_id,
    disabled,
    open,
    ..
  } = use_context::<CollapsibleContextValue>()
    .expect("CollapsibleContentImpl must be used in a CollapsibleRoot component");

  let is_open = Signal::derive(move || open.get() || is_present.get());
  let is_mount_animation_prevented = StoredValue::new(is_open.get_untracked());

  let original_styles = StoredValue::<Option<CssStyleDeclaration>>::new(None);
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

  let rect_size = Signal::derive(move || {
    let mut node = node_ref.get()?;
    let node_style = window().get_computed_style(&node).ok()?;

    if original_styles.get_value().is_none() {
      let new_styles = CssStyleDeclaration::from(JsValue::from(Object::new()));

      if let Some(node_style) = node_style {
        if let Ok(transition_duration) = node_style.get_property_value("transition-duration") {
          _ = new_styles.set_property("transition-duration", &transition_duration);
        }

        if let Ok(animation_name) = node_style.get_property_value("animation-name") {
          _ = new_styles.set_property("animation-name", &animation_name);
        }
      }

      original_styles.set_value(Some(new_styles));
    }

    logging::log!("removing animations");
    node = node
      .style("transition-duration", "0s")
      .style("animation-name", "none");

    let rect = node.get_bounding_client_rect();

    if !is_mount_animation_prevented.get_value() {
      logging::log!("adding back animations");

      _ = node
        .style(
          "transition-duration",
          original_styles
            .get_value()
            .and_then(|styles| styles.get_property_value("transition-duration").ok()),
        )
        .style(
          "animation-name",
          original_styles
            .get_value()
            .and_then(|styles| styles.get_property_value("animation-name").ok()),
        );
    }

    logging::log!("{} {}", rect.width(), rect.height());

    Some((rect.width(), rect.height()))
  });

  let present_state = Signal::derive(move || {
    rect_size
      .get()
      .map(|_| is_present.get())
      .unwrap_or(is_present.get())
  });

  Effect::new(move |_| {
    let Some(node) = node_ref.get() else {
      return;
    };

    let Some((width, height)) = rect_size.get() else {
      return;
    };

    _ = node
      .style(
        "--primitive-collapsible-content-width",
        format!("{width}px"),
      )
      .style(
        "--primitive-collapsible-content-height",
        format!("{height}px"),
      );
  });

  let children = StoredValue::new(children);

  view! {
    <Primitive
      {..attrs}
      attr:data-state=move || {
        if open.get() {
          "open"
        } else {
          "closed"
        }
      }
      attr:data-disabled=disabled
      attr:id=content_id
      attr:hidden=move || !(is_open.get() || present_state.get())
      element=html::div
      node_ref=node_ref
      as_child=as_child
    >
      <Show when=move || is_open.get()>
        {children.with_value(|children| children())}
      </Show>
    </Primitive>
  }
}
