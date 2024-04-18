use leptos::{html::AnyElement, *};
use wasm_bindgen::JsCast;
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
  disabled: Signal<Option<bool>>,
  open: Signal<bool>,
  on_open_toggle: Callback<()>,
}

#[component]
pub fn CollapsibleRoot(
  #[prop(optional)] open: Option<MaybeSignal<bool>>,
  #[prop(optional)] default_open: Option<MaybeSignal<bool>>,
  #[prop(optional)] disabled: Option<MaybeSignal<bool>>,
  #[prop(optional)] on_open_change: Option<Callback<bool>>,
  #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
  #[prop(optional)] as_child: Option<bool>,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: Children,
) -> impl IntoView {
  let (open, set_open) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || open.map(|open| open.get())),
    default_value: Signal::derive(move || default_open.map(|default_open| default_open.get())),
    on_change: Callback::new(move |value| {
      if let Some(on_open_change) = on_open_change {
        on_open_change(value);
      }
    }),
  });

  provide_context(CollapsibleContextValue {
    open: Signal::derive(move || open.get().unwrap_or(false)),
    disabled: Signal::derive(move || disabled.map(|disabled| disabled.get())),
    content_id: create_id(),
    on_open_toggle: Callback::new(move |_| {
      set_open.update(|open| *open = Some(!open.unwrap_or(false)))
    }),
  });

  let mut merged_attrs = vec![
    (
      "data-state",
      Signal::derive(move || {
        if open.get().unwrap_or(false) {
          "open"
        } else {
          "closed"
        }
      })
      .into_attribute(),
    ),
    (
      "data-disabled",
      Signal::derive(move || disabled.map(|disabled| disabled.get()).unwrap_or(false))
        .into_attribute(),
    ),
  ];

  merged_attrs.extend(attrs.into_iter());

  view! {
    <Primitive
      element=html::div
      node_ref=node_ref
      as_child=as_child
      attrs=merged_attrs
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn CollapsibleTrigger(
  #[prop(optional)] as_child: Option<bool>,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
  #[prop(attrs)] attrs: Attributes,
  children: Children,
) -> impl IntoView {
  let CollapsibleContextValue {
    content_id,
    disabled,
    open,
    on_open_toggle,
  } = use_context::<CollapsibleContextValue>()
    .expect("CollapsibleTrigger must be used in a CollapsibleRoot component");

  let mut merged_attrs = vec![
    (
      "aria-controls",
      Signal::derive(move || content_id.get()).into_attribute(),
    ),
    (
      "aria-expanded",
      Signal::derive(move || open.get()).into_attribute(),
    ),
    (
      "data-state",
      Signal::derive(move || if open.get() { "open" } else { "closed" }).into_attribute(),
    ),
    (
      "data-disabled",
      Signal::derive(move || disabled.get().unwrap_or(false)).into_attribute(),
    ),
    (
      "disabled",
      Signal::derive(move || disabled.get().unwrap_or(false)).into_attribute(),
    ),
  ];

  merged_attrs.extend(attrs.into_iter());

  view! {
    <Primitive
      element=html::button
      attrs=merged_attrs
      node_ref=node_ref
      as_child=as_child
      on:click=move |ev: MouseEvent| {
        if let Some(on_click) = on_click {
          on_click(ev);
        }

        on_open_toggle(());
      }
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn CollapsibleContent(
  #[prop(optional)] force_mount: Option<MaybeSignal<bool>>,
  #[prop(optional)] as_child: Option<bool>,
  // #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,
) -> impl IntoView {
  let CollapsibleContextValue {
    content_id,
    disabled,
    open,
    ..
  } = use_context::<CollapsibleContextValue>()
    .expect("CollapsibleContent must be used in a CollapsibleRoot component");

  let is_present = Signal::derive(move || {
    open.get()
      || force_mount
        .map(|force_mount| force_mount.get())
        .unwrap_or(false)
  });

  // let presence = create_presence(is_present);
  // let (present_state, set_present_state) = create_signal(presence.is_present.get());

  let node_ref = NodeRef::<AnyElement>::new();

  let is_open = Signal::derive(move || open.get() || is_present.get());
  let is_mount_animation_prevented = StoredValue::new(is_open.get());
  let original_styles = StoredValue::<Option<CssStyleDeclaration>>::new(None);

  Effect::new(move |_| {
    let Ok(animation_frame) = request_animation_frame_with_handle(move || {
      is_mount_animation_prevented.set_value(false);
    }) else {
      return;
    };

    on_cleanup(move || {
      animation_frame.cancel();
    });
  });

  let rect_size = Signal::derive(move || {
    let node = node_ref.get()?;

    let foob = node.get_bounding_client_rect();

    logging::log!("foob: {} {}", foob.width(), foob.height());

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

    _ = node
      .clone()
      .style("transition-duration", "0s")
      .style("animation-name", "none");

    let rect = node.get_bounding_client_rect();

    if is_mount_animation_prevented.get_value() == false {
      _ = node
        .style(
          "transition-duration",
          original_styles
            .get_value()
            .map(|styles| styles.get_property_value("transition-duration").ok())
            .flatten(),
        )
        .style(
          "animation-name",
          original_styles
            .get_value()
            .map(|styles| styles.get_property_value("animation-name").ok())
            .flatten(),
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
  // let (present_state, set_present_state) = create_signal(is_present.get());

  view! {
    // {move || presence.is_present.get().then(|| {
    {move || is_present.get().then(|| {
      let mut merged_attrs = vec![
        (
          "data-state",
          Signal::derive(move || {
            if open.get() {
              "open"
            } else {
              "closed"
            }
          })
          .into_attribute(),
        ),
        (
          "data-disabled",
          Signal::derive(move || disabled.get().unwrap_or(false))
            .into_attribute()
        ),
        ("id", Signal::derive(move || content_id.get()).into_attribute()),
        ("fart", "master".into_attribute()),
        ("hidden", Signal::derive(move || !(is_open.get() || present_state.get())).into_attribute()),
        ("style", Signal::derive(move || {
          let Some((width, height)) = rect_size.get() else {
            return String::new();
          };

          format!("{}{}",
            format!("--primitive-collapsible-content-height: {height}px; "),
            format!("--primitive-collapsible-content-width: {width}px"),
          )
        }).into_attribute())
      ];

      merged_attrs
        .extend(
          attrs
            .clone()
            .into_iter()
            .map(|(name, attr)| {
              if name == "style" {
                let attr = Signal::derive(move || {
                  let old_attr = attr.as_nameless_value_string().map(|value| format!("{value}; ")).unwrap_or_default();

                  let Some((width, height)) = rect_size.get() else {
                    return old_attr;
                  };

                  format!("{old_attr}{}{}",
                    format!("--primitive-collapsible-content-height: {height}px; "),
                    format!("--primitive-collapsible-content-width: {width}px"),
                  )
                });

                (name, attr.into_attribute())
              } else {
                (name, attr)
              }
            })
        );

      let children = children.clone();

      view! {
        <Primitive
          element=html::div
          attrs=merged_attrs
          as_child=as_child
          node_ref=node_ref
          on:click=move |e| {
            let foo = e.target().unwrap();
            let bar = foo.dyn_ref::<web_sys::HtmlDivElement>().unwrap();
            let baz = bar.get_bounding_client_rect();

            let fizz = node_ref.get().unwrap();
            let buzz = fizz.get_bounding_client_rect();

            logging::log!("foo:  {} {}", baz.width(), baz.height());
            logging::log!("fizz: {} {}", buzz.width(), buzz.height());
          }
        >
          {move || (is_open.get() || present_state.get()).then(|| children())}
        </Primitive>
      }
    })}
  }
}