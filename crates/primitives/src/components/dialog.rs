use leptos::{html::AnyElement, *};
use leptos_use::use_document;
use wasm_bindgen::JsCast;

use crate::{
  components::{presence::create_presence, primitive::Primitive},
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    create_id::create_id,
  },
  Attributes,
};

#[derive(Clone)]
struct DialogContextValue {
  trigger_ref: NodeRef<AnyElement>,
  content_ref: NodeRef<AnyElement>,
  content_id: Signal<String>,
  title_id: Signal<String>,
  description_id: Signal<String>,
  open: Signal<bool>,
  on_open_change: Callback<bool>,
  on_open_toggle: Callback<()>,
  modal: Signal<bool>,
}

#[component]
pub fn DialogRoot(
  #[prop(optional)] open: Option<MaybeSignal<bool>>,
  #[prop(optional)] default_open: Option<MaybeSignal<bool>>,
  #[prop(optional)] modal: Option<MaybeSignal<bool>>,
  #[prop(optional)] on_open_change: Option<Callback<bool>>,
  children: Children,
) -> impl IntoView {
  let (open, set_open) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || open.map(|open| open.get())),
    default_value: Signal::derive(move || default_open.map(|default_open| default_open.get())),
    on_change: Callback::new(move |changed| {
      if let Some(on_open_change) = on_open_change {
        on_open_change(changed);
      }
    }),
  });

  provide_context(DialogContextValue {
    trigger_ref: NodeRef::new(),
    content_ref: NodeRef::new(),
    content_id: create_id(),
    title_id: create_id(),
    description_id: create_id(),
    open: Signal::derive(move || open.get().unwrap_or(false)),
    on_open_change: Callback::new(move |changed| set_open.set(changed)),
    on_open_toggle: Callback::new(move |_| {
      set_open.update(|open| *open = Some(!(*open).unwrap_or(false)));
    }),
    modal: Signal::derive(move || modal.map(|modal| modal.get()).unwrap_or(true)),
  });

  view! {
      {children()}
  }
}

#[component]
pub fn DialogTrigger(#[prop(attrs)] attributes: Attributes, children: Children) -> impl IntoView {
  let DialogContextValue {
    open,
    content_id,
    trigger_ref,
    on_open_toggle,
    ..
  } = use_context().expect("DialogTrigger must be used in a DialogRoot component");

  let mut merged_attrs = vec![
    ("type", "button".into_attribute()),
    ("aria-haspopup", "dialog".into_attribute()),
    ("aria-expanded", (move || open.get()).into_attribute()),
    ("aria-controls", (move || content_id.get()).into_attribute()),
    (
      "data-state",
      (move || if open.get() { "open" } else { "closed" }).into_attribute(),
    ),
  ];

  merged_attrs.extend(attributes);

  view! {
      <Primitive
          element=html::button
          attrs=merged_attrs
          node_ref=trigger_ref
          on:click=move |_| {
              on_open_toggle(());
          }
      >
        {children()}
      </Primitive>
  }
}

#[derive(Clone)]
struct PortalContextValue {
  force_mount: Option<Signal<bool>>,
}

#[component]
pub fn DialogPortal(
  #[prop(optional)] container: Option<NodeRef<AnyElement>>,
  #[prop(optional)] force_mount: Option<MaybeSignal<bool>>,
  children: ChildrenFn,
) -> impl IntoView {
  let DialogContextValue { open, .. } =
    use_context().expect("DialogPortal must be used in a Dialog component");

  let is_present = Signal::derive(move || {
    force_mount
      .map(|force_mount| force_mount.get())
      .unwrap_or(false)
      || open.get()
  });

  provide_context(PortalContextValue {
    force_mount: force_mount.map(|force_mount| force_mount.into_signal()),
  });

  children()
    .nodes
    .into_iter()
    .map(|child| {
      let child = StoredValue::new(child);

      let node_ref = NodeRef::<AnyElement>::new();
      let presence = create_presence(is_present, node_ref);

      let container = move || {
        container
          .map(|container| {
            container.get().and_then(|container| {
              container
                .dyn_ref::<web_sys::HtmlElement>()
                .map(|container| (*container).clone())
            })
          })
          .unwrap_or(
            use_document()
              .as_ref()
              .and_then(|document| Some(document.body()?.clone())),
          )
      };

      view! {
        <Show when=presence>
          {move || {
              match container() {
                  Some(container) => {
                      view! {
                        <Portal mount=container>
                            <Primitive
                                element=html::div
                                as_child=Some(true)
                                node_ref=node_ref
                            >
                                {child.with_value(|child| child.clone())}
                            </Primitive>
                        </Portal>
                      }
                  },
                  None => ViewFn::default().run()
              }
          }}
        </Show>
      }
    })
    .collect_view()
}

#[component]
pub fn DialogOverlay(
  #[prop(optional)] force_mount: Option<MaybeSignal<bool>>,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,
) -> impl IntoView {
  let PortalContextValue {
    force_mount: portal_force_mount,
  } = use_context().expect("DialogOverlay must be used in a Portal component");

  let DialogContextValue { open, modal, .. } =
    use_context().expect("DialogOverlay must be used in a DialogRoot component");

  let force_mount = Signal::derive(move || {
    force_mount
      .map(|force_mount| force_mount.into_signal())
      .or(portal_force_mount)
      .map(|force_mount| force_mount.get())
      .unwrap_or(false)
  });

  let children = StoredValue::new(children);
  let attrs = StoredValue::new(attrs);

  view! {
      <Show when=modal>
        {move || {
            let is_present = Signal::derive(move || force_mount.get() || open.get());
            let presence = create_presence(is_present, node_ref);

            view! {
                <Show when=presence>
                    <DialogOverlayImpl
                        node_ref=node_ref
                        attrs=attrs.get_value()
                    >
                        {children.with_value(|children| children())}
                    </DialogOverlayImpl>
                </Show>
            }
        }}
      </Show>
  }
}

#[component]
fn DialogOverlayImpl(
  node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,
) -> impl IntoView {
  let DialogContextValue {
    content_ref, open, ..
  } = use_context().expect("DialogOverlayImpl must be used in a Dialog component");

  let mut merged_attrs = vec![(
    "data-state",
    (move || if open.get() { "open" } else { "closed" }).into_attribute(),
  )];

  merged_attrs.extend(attrs);

  Effect::new(move |_| {
    let Some(node) = node_ref.get() else {
      return;
    };

    _ = node.style("pointer-events", "auto");
  });

  // TODO: add RemoveScroll here
  view! {
      <Primitive
          element=html::div
          node_ref=node_ref
          attrs=merged_attrs
      >
          {children()}
      </Primitive>
  }
}

#[component]
pub fn DialogContent(
  #[prop(optional)] force_mount: Option<MaybeSignal<bool>>,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: ChildrenFn,
) -> impl IntoView {
  let PortalContextValue {
    force_mount: portal_force_mount,
  } = use_context().expect("DialogOverlay must be used in a Portal component");

  let DialogContextValue { open, modal, .. } =
    use_context().expect("DialogOverlay must be used in a DialogRoot component");

  let force_mount = Signal::derive(move || {
    force_mount
      .map(|force_mount| force_mount.into_signal())
      .or(portal_force_mount)
      .map(|force_mount| force_mount.get())
      .unwrap_or(false)
  });

  let is_present = Signal::derive(move || force_mount.get() || open.get());
  let presence = create_presence(is_present, node_ref);

  let children = StoredValue::new(children);

  view! {
      <Show when=presence>
        {move || {
            if modal.get() {
                view! {
                    <DialogContentModal
                        node_ref=node_ref
                    >
                        {children.with_value(|children| children())}
                    </DialogContentModal>
                }
            } else {
                view! {
                    <DialogContentNonModal
                        node_ref=node_ref
                    >
                        {children.with_value(|children| children())}
                    </DialogContentNonModal>
                }
            }
        }}
      </Show>
  }
}

#[component]
fn DialogContentModal(
  node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,
) -> impl IntoView {
  let DialogContextValue {
    open,
    trigger_ref,
    content_ref,
    ..
  } = use_context().expect("DialogContentModal must be used in a DialogRoot component");

  Effect::new(move |_| {
    if let Some(node) = content_ref.get() {
      // TODO: add aria-hidden
      // hide_others(node);
    }
  });

  view! {
      <DialogContentImpl
          attrs=attrs
          node_ref=content_ref
          trap_focus=open.into()
          disable_outside_pointer_events=true.into()
          on_close_auto_focus=Callback::new(move |ev: web_sys::Event| {
              ev.prevent_default();
              if let Some(trigger) = trigger_ref.get() {
                  trigger.focus();
              }
          })
          on_pointer_down_outside=Callback::new(move |ev: web_sys::Event| {
              //let original_ev = ev.detail().original_event();
              //let ctrl_left_click = original_ev.button() == 0 && original_ev.ctrl_key() == true;
              //let is_right_click = original_ev.button() == 2 || ctrl_left_click;

              //if is_right_click {
              //    ev.prevent_default();
              //}
          })
          on_focus_outside=Callback::new(move |ev: web_sys::FocusEvent| {
              ev.prevent_default();
          })
      >
        {children()}
      </DialogContentImpl>
  }
}

#[component]
fn DialogContentNonModal(
  node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,
  #[prop(optional)] on_close_auto_focus: Option<Callback<web_sys::Event>>,
  #[prop(optional)] on_interact_outside: Option<Callback<web_sys::Event>>,
) -> impl IntoView {
  let DialogContextValue {
    trigger_ref,
    content_ref,
    ..
  } = use_context().expect("DialogContentModal must be used in a DialogRoot component");

  let has_interacted_outside = StoredValue::new(false);
  let has_pointer_down_outside = StoredValue::new(false);

  Effect::new(move |_| {
    if let Some(node) = content_ref.get() {
      // TODO: add aria-hidden
      // hide_others(node);
    }
  });

  view! {
      <DialogContentImpl
          attrs=attrs
          node_ref=content_ref
          trap_focus=false.into()
          disable_outside_pointer_events=false.into()
          on_close_auto_focus=Callback::new(move |ev: web_sys::Event| {
              if let Some(on_close_auto_focus) = on_close_auto_focus {
                  on_close_auto_focus(ev.clone());
              }

              if !ev.default_prevented() {
                  if !has_interacted_outside.get_value() {
                      if let Some(trigger) = trigger_ref.get() {
                          trigger.focus();
                      }
                  }

                  ev.prevent_default();
              }

              has_interacted_outside.set_value(false);
              has_pointer_down_outside.set_value(false);
          })
          on_interact_outside=Callback::new(move |ev: web_sys::Event| {
              if let Some(on_interact_outside) = on_interact_outside {
                  on_interact_outside(ev.clone());
              }

              if !ev.default_prevented() {
                  has_interacted_outside.set_value(true);

                  //if ev.detail().original_event().type_() == "pointerdown" {
                  //    has_pointer_down_outside.set_value(true);
                  //}
              }

              let Some(target) = ev.target() else {
                  return;
              };

              let Some(target) = target.dyn_ref::<web_sys::HtmlElement>() else {
                  return;
              };

              let target_is_trigger = trigger_ref.get().map(|trigger| trigger.contains(Some(target))).unwrap_or(false);
              if target_is_trigger {
                  ev.prevent_default();
              }

              //if ev.detail().original_event().type_() == "focusin" && has_pointer_down_outside.get_value() {
              //    ev.prevent_default();
              //}
          })
          on_focus_outside=Callback::new(move |ev: web_sys::FocusEvent| {
              ev.prevent_default();
          })
      >
        {children()}
      </DialogContentImpl>
  }
}

#[component]
fn DialogContentImpl(
  #[prop(attrs)] attrs: Attributes,
  node_ref: NodeRef<AnyElement>,
  trap_focus: MaybeSignal<bool>,
  disable_outside_pointer_events: MaybeSignal<bool>,
  children: Children,
  on_close_auto_focus: Callback<web_sys::Event>,
  #[prop(optional)] on_pointer_down_outside: Option<Callback<web_sys::Event>>,
  on_focus_outside: Callback<web_sys::FocusEvent>,
  #[prop(optional)] on_interact_outside: Option<Callback<web_sys::Event>>,
) -> impl IntoView {
  view! {}
}

#[component]
pub fn DialogTitle() -> impl IntoView {
  view! {}
}

#[component]
pub fn DialogClose() -> impl IntoView {
  view! {}
}

#[component]
fn TitleWarning() -> impl IntoView {
  view! {}
}

#[component]
fn DescriptionWarning() -> impl IntoView {
  view! {}
}
