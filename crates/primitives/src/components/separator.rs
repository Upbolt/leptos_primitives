use leptos::{html::AnyElement, *};

use crate::{components::primitive::Primitive, util::Orientation, Attributes};

#[component]
pub fn SeparatorRoot(
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] decorative: MaybeSignal<bool>,
  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
) -> impl IntoView {
  let mut merged_attrs = if decorative.get_untracked() {
    vec![("role", "none".into_attribute())]
  } else {
    vec![
      (
        "aria-orientation",
        Signal::derive(move || orientation.get().to_string()).into_attribute(),
      ),
      ("role", "separator".into_attribute()),
    ]
  };

  merged_attrs.extend(attrs);
  merged_attrs.extend([(
    "data-orientation",
    Signal::derive(move || orientation.get().to_string()).into_attribute(),
  )]);

  view! {
    <Primitive
      element=html::div
      attrs=merged_attrs
      node_ref=node_ref
    >
      {().into_view()}
    </Primitive>
  }
}
