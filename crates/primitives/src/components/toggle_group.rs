use leptos::{html::AnyElement, *};

use crate::{
  components::{
    primitive::Primitive,
    roving_focus::{RovingFocusGroup, RovingFocusGroupItem},
    toggle::ToggleRoot,
  },
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    Direction, Orientation,
  },
  Attributes,
};

pub enum ToggleGroupKind {
  Single {
    value: MaybeProp<String>,
    default_value: MaybeProp<String>,
    on_value_change: Option<Callback<String>>,
  },
  Multiple {
    value: MaybeProp<Vec<String>>,
    default_value: MaybeProp<Vec<String>>,
    on_value_change: Option<Callback<Vec<String>>>,
  },
}

pub struct ToggleGroupSingle;
pub struct ToggleGroupMultiple;

impl ToggleGroupSingle {
  pub fn none() -> Option<String> {
    None
  }
}

impl ToggleGroupMultiple {
  pub fn none() -> Option<Vec<String>> {
    None
  }
}

#[component]
pub fn ToggleGroupRoot(
  kind: ToggleGroupKind,

  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(default=true.into(), into)] roving_focus: MaybeSignal<bool>,
  #[prop(default=true.into(), into)] should_loop: MaybeSignal<bool>,
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] direction: MaybeSignal<Direction>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  match kind {
    ToggleGroupKind::Single {
      value,
      default_value,
      on_value_change,
    } => view! {
      <ToggleGroupSingleImpl
        disabled=disabled
        roving_focus=roving_focus
        should_loop=should_loop
        orientation=orientation
        direction=direction
        value=value
        default_value=default_value
        on_value_change=on_value_change.unwrap_or((|_|{}).into())
        node_ref=node_ref
        attrs=attrs
        as_child=as_child
      >
        {children()}
      </ToggleGroupSingleImpl>
    },
    ToggleGroupKind::Multiple {
      value,
      default_value,
      on_value_change,
    } => view! {
      <ToggleGroupMultipleImpl
        disabled=disabled
        roving_focus=roving_focus
        should_loop=should_loop
        orientation=orientation
        direction=direction
        value=value
        default_value=default_value
        on_value_change=on_value_change.unwrap_or((|_|{}).into())
        node_ref=node_ref
        attrs=attrs
        as_child=as_child
      >
        {children()}
      </ToggleGroupMultipleImpl>
    },
  }
}

#[derive(Clone, PartialEq)]
enum ToggleGroupValueKind {
  Single,
  Multiple,
}

#[derive(Clone)]
struct ToggleGroupValueContextValue {
  kind: ToggleGroupValueKind,
  value: Signal<Vec<String>>,
  on_item_activate: Callback<String>,
  on_item_deactivate: Callback<String>,
}

#[component]
fn ToggleGroupSingleImpl(
  disabled: MaybeSignal<bool>,
  roving_focus: MaybeSignal<bool>,
  should_loop: MaybeSignal<bool>,
  orientation: MaybeSignal<Orientation>,
  direction: MaybeSignal<Direction>,
  #[prop(optional, into)] value: MaybeProp<String>,
  #[prop(optional, into)] default_value: MaybeProp<String>,

  on_value_change: Callback<String>,

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

  let set_on_item_activate = set_value.clone();
  let set_on_item_deactivate = set_value.clone();
  provide_context(ToggleGroupValueContextValue {
    kind: ToggleGroupValueKind::Single,
    value: Signal::derive(move || value.get().map(|value| vec![value]).unwrap_or_default()),
    on_item_activate: Callback::new(move |item| {
      set_on_item_activate.set(item);
    }),
    on_item_deactivate: Callback::new(move |_| set_on_item_deactivate.set(String::new())),
  });

  view! {
    <ToggleGroup
      disabled=disabled
      roving_focus=roving_focus
      should_loop=should_loop
      orientation=orientation
      direction=direction
      node_ref=node_ref
      attrs=attrs
      as_child=as_child
    >
      {children()}
    </ToggleGroup>
  }
}

#[component]
fn ToggleGroupMultipleImpl(
  disabled: MaybeSignal<bool>,
  roving_focus: MaybeSignal<bool>,
  should_loop: MaybeSignal<bool>,
  orientation: MaybeSignal<Orientation>,
  direction: MaybeSignal<Direction>,
  #[prop(optional, into)] value: MaybeProp<Vec<String>>,
  #[prop(optional, into)] default_value: MaybeProp<Vec<String>>,

  on_value_change: Callback<Vec<String>>,

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

  let set_on_item_activate = set_value.clone();
  let set_on_item_deactivate = set_value.clone();
  provide_context(ToggleGroupValueContextValue {
    kind: ToggleGroupValueKind::Multiple,
    value: Signal::derive(move || value.get().unwrap_or_default()),
    on_item_activate: Callback::new(move |item| {
      set_on_item_activate.update(|value| {
        if let Some(value) = value {
          value.push(item);
        } else {
          *value = Some(vec![item]);
        }
      });
    }),
    on_item_deactivate: Callback::new(move |item| {
      set_on_item_deactivate.update(|value| {
        if let Some(value) = value {
          *value = value
            .iter()
            .filter_map(|value| (*value != item).then_some(value.to_string()))
            .collect::<Vec<_>>();
        } else {
          *value = Some(vec![]);
        }
      })
    }),
  });

  view! {
    <ToggleGroup
      disabled=disabled
      roving_focus=roving_focus
      should_loop=should_loop
      orientation=orientation
      direction=direction
      node_ref=node_ref
      attrs=attrs
      as_child=as_child
    >
      {children()}
    </ToggleGroup>
  }
}

#[derive(Clone)]
struct ToggleGroupStateContextValue {
  disabled: Signal<bool>,
  roving_focus: Signal<bool>,
}

#[component]
fn ToggleGroup(
  disabled: MaybeSignal<bool>,
  roving_focus: MaybeSignal<bool>,
  should_loop: MaybeSignal<bool>,
  orientation: MaybeSignal<Orientation>,
  direction: MaybeSignal<Direction>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  provide_context(ToggleGroupStateContextValue {
    roving_focus: Signal::derive(move || roving_focus.get()),
    disabled: Signal::derive(move || disabled.get()),
  });

  let children = StoredValue::new(children);
  let attrs = StoredValue::new(attrs);

  view! {
    <Show
      when=move || roving_focus.get()
      fallback=move || view! {
        <Primitive
          {..attrs.get_value()}
          attr:role="group"
          attr:dir=move || direction.get().to_string()
          element=html::div
          node_ref=node_ref
          as_child=as_child
        >
          {children.with_value(|children| children())}
        </Primitive>
      }
    >
      <RovingFocusGroup
        as_child=true
        orientation=Signal::derive(move || orientation.get())
        direction=Signal::derive(move || direction.get())
        should_loop=Signal::derive(move || should_loop.get())
      >
        <Primitive
          {..attrs.get_value()}
          attr:role="group"
          attr:dir=move || direction.get().to_string()
          element=html::div
          node_ref=node_ref
          as_child=as_child
        >
          {children.with_value(|children| children())}
        </Primitive>
      </RovingFocusGroup>
    </Show>
  }
}

#[component]
pub fn ToggleGroupItem(
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(into)] value: MaybeSignal<String>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let ToggleGroupValueContextValue {
    kind,
    value: context_value,
    on_item_activate,
    on_item_deactivate,
  } = use_context().expect("ToggleGroupItem must be in a ToggleGroupRoot component");
  let ToggleGroupStateContextValue {
    disabled: context_disabled,
    roving_focus,
  } = use_context().expect("ToggleGroupItem must be in a ToggleGroupRoot component");

  let is_pressed_value = value.clone();
  let is_pressed = Signal::derive(move || context_value.get().contains(&is_pressed_value.get()));
  let is_disabled = Signal::derive(move || context_disabled.get() || disabled.get());
  let focusable = Signal::derive(move || !is_disabled.get());

  let children = StoredValue::new(children);
  let kind = StoredValue::new(kind);
  let attrs = StoredValue::new(attrs);
  let value = StoredValue::new(value);

  view! {
    <Show
      when=move || roving_focus.get()
      fallback=move || view! {
        <ToggleRoot
          {..attrs.get_value()}
          attr:role=move || (kind.get_value() == ToggleGroupValueKind::Single).then_some("radio")
          attr:aria-checked=move || (kind.get_value() == ToggleGroupValueKind::Single).then_some(is_pressed.get().to_string())
          disabled=is_disabled
          pressed=is_pressed
          on_pressed_changed=Callback::new(move |pressed| {
            if pressed {
              on_item_activate.call(value.get_value().get());
            } else {
              on_item_deactivate.call(value.get_value().get());
            }
          })
          node_ref=node_ref
          as_child=as_child
        >
          {children.with_value(|children| children())}
        </ToggleRoot>
      }
    >
      <RovingFocusGroupItem
        as_child=true
        focusable=focusable
        active=is_pressed
      >
        <ToggleRoot
          {..attrs.get_value()}
          attr:role=move || (kind.get_value() == ToggleGroupValueKind::Single).then_some("radio")
          attr:aria-checked=move || (kind.get_value() == ToggleGroupValueKind::Single).then_some(is_pressed.get().to_string())
          disabled=is_disabled
          pressed=is_pressed
          on_pressed_changed=Callback::new(move |pressed| {
            if pressed {
              on_item_activate.call(value.get_value().get());
            } else {
              on_item_deactivate.call(value.get_value().get());
            }
          })
          node_ref=node_ref
          as_child=as_child
        >
          {children.with_value(|children| children())}
        </ToggleRoot>
      </RovingFocusGroupItem>
    </Show>
  }
}
