use leptos::{html::AnyElement, *};
use wasm_bindgen::JsCast;
use web_sys::{HtmlAnchorElement, KeyboardEvent};

use crate::{
  components::{
    primitive::Primitive,
    roving_focus::{RovingFocusGroup, RovingFocusGroupItem},
    separator::SeparatorRoot,
    toggle_group::{ToggleGroupItem, ToggleGroupRoot},
  },
  util::{Direction, Orientation},
  Attributes,
};

use super::toggle_group::ToggleGroupKind;

#[derive(Clone)]
struct ToolbarContextValue {
  orientation: Signal<Orientation>,
  direction: Signal<Direction>,
}

#[component]
pub fn ToolbarRoot(
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] direction: MaybeSignal<Direction>,
  #[prop(default=true.into(), into)] should_loop: MaybeSignal<bool>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  provide_context(ToolbarContextValue {
    orientation: Signal::derive(move || orientation.get()),
    direction: Signal::derive(move || direction.get()),
  });

  let children = StoredValue::new(children);

  view! {
    <RovingFocusGroup
      orientation=Signal::derive(move || orientation.get())
      direction=Signal::derive(move || direction.get())
      should_loop=Signal::derive(move || should_loop.get())
    >
      <Primitive
        {..attrs}
        attr:role="toolbar"
        attr:aria-orientation=move || orientation.get().to_string()
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
pub fn ToolbarSeparator(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] children: Option<ChildrenFn>,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let ToolbarContextValue { orientation, .. } =
    use_context().expect("ToolbarSeparator must be used in a ToolbarRoot component");

  let orientation = Signal::derive(move || match orientation.get() {
    Orientation::Horizontal => Orientation::Vertical,
    Orientation::Vertical => Orientation::Horizontal,
  });

  let children = StoredValue::new(children);

  view! {
    <SeparatorRoot
      orientation=orientation
      node_ref=node_ref
      attrs=attrs
      as_child=as_child
    >
      {children.with_value(|children| children.as_ref().map(|children| children()))}
    </SeparatorRoot>
  }
}

#[component]
pub fn ToolbarButton(
  #[prop(optional, into)] disabled: MaybeSignal<bool>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let children = StoredValue::new(children);

  view! {
    <RovingFocusGroupItem
      as_child=true
      focusable=Signal::derive(move || disabled.get())
    >
      <Primitive
        {..attrs}
        attr:type="button"
        element=html::button
        node_ref=node_ref
        as_child=as_child
      >
        {children.with_value(|children| children())}
      </Primitive>
    </RovingFocusGroupItem>
  }
}

#[component]
pub fn ToolbarLink(
  #[prop(default=(|_|{}).into(), into)] on_key_down: Callback<KeyboardEvent>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let children = StoredValue::new(children);

  view! {
    <RovingFocusGroupItem
      as_child=true
      focusable=true
    >
      <Primitive
        element=html::a
        node_ref=node_ref
        attrs=attrs.clone()
        as_child=as_child
        on:keydown=move |ev: KeyboardEvent| {
          on_key_down.call(ev.clone());

          if ev.key() == " " {
            if let Some(current_target) = ev.current_target() {
              if let Some(current_target) = current_target.dyn_ref::<HtmlAnchorElement>() {
                current_target.click();
              }
            }
          }
        }
      >
        {children.with_value(|children| children())}
      </Primitive>
    </RovingFocusGroupItem>
  }
}

#[component]
pub fn ToolbarToggleGroup(
  kind: ToggleGroupKind,

  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] direction: MaybeSignal<Direction>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let context = use_context::<ToolbarContextValue>()
    .expect("ToolbarToggleGroup must be in a ToolbarRoot component");

  view! {
    <ToggleGroupRoot
      {..attrs}
      attr:data-orientation=move || context.orientation.get().to_string()
      attr:dir=move || context.direction.get().to_string()
      kind=kind
      disabled=Signal::derive(move || disabled.get())
      orientation=Signal::derive(move || orientation.get())
      direction=Signal::derive(move || direction.get())
      roving_focus=false
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </ToggleGroupRoot>
  }
}

#[component]
pub fn ToolbarToggleItem(
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(into)] value: MaybeSignal<String>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let children = StoredValue::new(children);

  view! {
    <ToolbarButton as_child=true>
      <ToggleGroupItem
        disabled=Signal::derive(move || disabled.get())
        value=value.clone()
        node_ref=node_ref
        attrs=attrs.clone()
        as_child=as_child
      >
        {children.with_value(|children| children())}
      </ToggleGroupItem>
    </ToolbarButton>
  }
}
