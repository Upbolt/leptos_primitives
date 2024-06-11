use leptos::{html::AnyElement, *};
use web_sys::MouseEvent;

use crate::{components::primitive::Primitive, Attributes};

#[component]
pub fn LabelRoot(
  #[prop(optional, into)] for_html: MaybeProp<String>,

  #[prop(default=(|_|{}).into(), into)] on_mouse_down: Callback<MouseEvent>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  view! {
    <Primitive
      {..attrs}
      attr:for=for_html
      element=html::label
      on:mousedown=move |ev: MouseEvent| {
        on_mouse_down.call(ev.clone());

        if ev.default_prevented() && ev.detail() > 1 {
          ev.prevent_default();
        }
      }
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}
