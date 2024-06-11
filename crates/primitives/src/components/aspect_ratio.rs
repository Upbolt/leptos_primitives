use leptos::{html::AnyElement, *};

use crate::{components::primitive::Primitive, Attributes};

#[component]
pub fn AspectRatioRoot(
  #[prop(default=1.0f64.into(), into)] ratio: MaybeSignal<f64>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  view! {
    <div
      style="position: relative; width: 100%;"
      style:padding-bottom=Signal::derive(move || format!("{}%", 100.0 / ratio.get()))
    >
      <Primitive
        {..attrs}
        attr:style="position: absolute; top: 0; right: 0; bottom: 0; left: 0"
        element=html::div
        node_ref=node_ref
        as_child=as_child
      >
        {children()}
      </Primitive>
    </div>
  }
}
