use leptos::{html::AnyElement, *};

use crate::{components::primitive::Primitive, util::Orientation, Attributes};

#[component]
pub fn SeparatorRoot(
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] decorative: MaybeSignal<bool>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] children: Option<ChildrenFn>,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let children = StoredValue::new(children);

  view! {
    <Primitive
      {..attrs}
      attr:role=move || if decorative.get() { "none" } else { "separator" }
      attr:aria-orientation=move || (!decorative.get()).then_some(orientation.get().to_string())
      attr:data-orientation=move || orientation.get().to_string()
      element=html::div
      node_ref=node_ref
      as_child=as_child
    >
      {children.with_value(|children| children.as_ref().map(|children| children()))}
    </Primitive>
  }
}
