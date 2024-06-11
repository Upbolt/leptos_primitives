use std::collections::HashMap;

use leptos::{
  ev::focus,
  html::{AnyElement, Input, Span},
  *,
};

use leptos_use::use_element_size;
use strum::EnumString;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
  js_sys::{Array, Function, JsString, Object, Reflect},
  DomRect, Event, EventInit, KeyboardEvent, PointerEvent,
};

use crate::{
  components::{
    collection::{use_collection_context, use_collection_item_ref, CollectionContextValue},
    primitive::Primitive,
  },
  util::{
    create_controllable_signal::{create_controllable_signal, CreateControllableSignalProps},
    create_previous::create_previous,
    linear_scale, Direction, Orientation,
  },
  Attributes,
};

#[derive(Clone)]
struct SliderContextValue {
  name: Signal<Option<String>>,
  disabled: Signal<bool>,
  min: Signal<f64>,
  max: Signal<f64>,
  values: Signal<Vec<f64>>,
  value_index_to_change: StoredValue<Option<usize>>,
  thumbs: StoredValue<Vec<HtmlElement<AnyElement>>>,
  orientation: Signal<Orientation>,
}

#[component]
pub fn SliderRoot(
  #[prop(optional, into)] name: MaybeProp<String>,
  #[prop(default=0.0f64.into(), into)] min: MaybeSignal<f64>,
  #[prop(default=100.0f64.into(), into)] max: MaybeSignal<f64>,
  #[prop(default=1.0f64.into(), into)] step: MaybeSignal<f64>,
  #[prop(optional, into)] orientation: MaybeSignal<Orientation>,
  #[prop(optional, into)] direction: MaybeSignal<Direction>,
  #[prop(optional, into)] disabled: MaybeSignal<bool>,
  #[prop(default=0.0f64.into(), into)] min_steps_between_thumbs: MaybeSignal<f64>,
  #[prop(optional, into)] value: MaybeProp<Vec<f64>>,
  #[prop(optional, into)] default_value: MaybeProp<Vec<f64>>,
  #[prop(optional, into)] inverted: MaybeSignal<bool>,

  #[prop(default=(|_|{}).into(), into)] on_value_change: Callback<Vec<f64>>,
  #[prop(default=(|_|{}).into(), into)] on_value_commit: Callback<Vec<f64>>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let thumbs = StoredValue::new(Vec::<HtmlElement<AnyElement>>::new());
  let value_index_to_change = StoredValue::new(Some(0usize));

  let (values, set_values) = create_controllable_signal(CreateControllableSignalProps {
    value: Signal::derive(move || value.get()),
    default_value: Signal::derive(move || Some(default_value.get().unwrap_or(vec![min.get()]))),
    on_change: Callback::new(move |value| {
      let thumbs = thumbs.get_value();
      let thumbs = Vec::from_iter(thumbs.iter());

      if let Some(value_index) = value_index_to_change.get_value() {
        if let Some(thumb) = thumbs.get(value_index) {
          _ = thumb.focus();
        }
      }

      on_value_change.call(value);
    }),
  });

  let values = Signal::derive(move || values.get().unwrap_or_default());
  let values_before_slide_start = StoredValue::new(values.get_untracked());

  let update_values = move |value: f64, at_index: usize, commit: bool| {
    let decimal_count = get_decimal_count(step.get());
    let snap_to_step = round_value(
      ((value - min.get()) / step.get()).round() * step.get() + min.get(),
      decimal_count as u32,
    );
    let next_value = snap_to_step.clamp(min.get(), max.get());

    set_values.update(move |values| {
      let previous_values = values.as_ref().cloned().unwrap_or_default();
      let next_values = get_next_sorted_values(&previous_values, next_value, at_index);

      if has_min_steps_between_values(&next_values, min_steps_between_thumbs.get() * step.get()) {
        value_index_to_change.set_value(next_values.iter().position(|value| value == &next_value));

        let updated_count = next_values
          .iter()
          .zip(previous_values.iter())
          .filter(|&(prev, curr)| prev == curr)
          .count();

        let has_changed =
          updated_count != next_values.len() || updated_count != previous_values.len();

        if has_changed {
          if commit {
            on_value_commit.call(next_values.clone());
          }

          *values = Some(next_values);
        }
      }
    });
  };

  let start_update = update_values.clone();
  let handle_slide_start = Callback::new(move |value: f64| {
    if let Some(closest_index) = find_closest_index(&values.get(), value) {
      start_update(value, closest_index, false);
    }
  });

  let move_update = update_values.clone();
  let handle_slide_move = Callback::new(move |value: f64| {
    if let Some(value_index) = value_index_to_change.get_value() {
      move_update(value, value_index, false);
    }
  });

  let handle_slide_end = Callback::new(move |_: ()| {
    let prev_value = value_index_to_change
      .get_value()
      .map(|index| values_before_slide_start.get_value().get(index).cloned())
      .flatten();

    let next_value = value_index_to_change
      .get_value()
      .map(|index| values.get().get(index).cloned())
      .flatten();

    let has_changed = next_value != prev_value;

    if has_changed {
      on_value_commit.call(values.get());
    }
  });

  provide_context(SliderContextValue {
    name: Signal::derive(move || name.get()),
    disabled: Signal::derive(move || disabled.get()),
    min: Signal::derive(move || min.get()),
    max: Signal::derive(move || max.get()),
    value_index_to_change,
    thumbs,
    values: Signal::derive(move || values.get()),
    orientation: Signal::derive(move || orientation.get()),
  });

  provide_context(CollectionContextValue::<SliderCollectionItem, AnyElement> {
    collection_ref: node_ref,
    item_map: RwSignal::new(HashMap::new()),
  });

  let home_key_down_update = update_values.clone();
  let end_key_down_update = update_values.clone();

  view! {
    <Slider
      {..attrs}
      attr:aria-disabled=disabled
      attr:data-disabled=move || disabled.get().then_some("")
      min=Signal::derive(move || min.get())
      max=Signal::derive(move || max.get())
      inverted=Signal::derive(move || inverted.get())
      direction=Signal::derive(move || direction.get())
      orientation=Signal::derive(move || orientation.get())
      on_slide_start=handle_slide_start
      on_slide_move=handle_slide_move
      on_slide_end=handle_slide_end
      on_home_key_down=Callback::new(move |_| {
        if !disabled.get() {
          home_key_down_update(min.get(), 0, true);
        }
      })
      on_end_key_down=Callback::new(move |_| {
        if !disabled.get() {
          end_key_down_update(max.get(), values.get().len() - 1, true);
        }
      })
      on_step_key_down=Callback::new(move |Step{ event, direction }| {
        if disabled.get() {
          return;
        }

        let is_page_key = ["PageUp", "PageDown"].contains(&event.key().as_str());
        let is_skip_key = is_page_key || (event.shift_key() && ["ArrowUp", "ArrowLeft", "ArrowRight", "ArrowDown"].contains(&event.key().as_str()));
        let multiplier = if is_skip_key { 10.0f64 } else { 1.0f64 };

        let Some(at_index) = value_index_to_change.get_value() else {
          return;
        };

        let value = values.get().get(at_index).cloned().unwrap_or(0.);
        let step_in_direction = step.get() * multiplier * match direction { OrientationDirection::Forward => 1.0f64, OrientationDirection::Backward => -1.0f64 };

        update_values(value + step_in_direction, at_index, true);
      })
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Slider>
  }
}

#[derive(Clone, PartialEq, EnumString, strum::Display, strum::IntoStaticStr)]
enum Side {
  Top,
  Right,
  Bottom,
  Left,
}

#[derive(Clone, PartialEq)]
enum OrientationDirection {
  Forward,
  Backward,
}

#[derive(Clone)]
enum Size {
  Width,
  Height,
}

#[derive(Clone)]
struct OrientationContextValue {
  start_edge: Signal<Side>,
  end_edge: Signal<Side>,
  size: Signal<Size>,
  direction: Signal<OrientationDirection>,
}

struct Step {
  event: KeyboardEvent,
  direction: OrientationDirection,
}
#[derive(Clone)]
enum SlideDirection {
  FromLeft,
  FromRight,
  FromBottom,
  FromTop,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct SliderCollectionItem;

#[derive(Clone)]
struct SliderImplContextValue {
  dom_rect: StoredValue<Option<DomRect>>,
}

#[component]
fn Slider(
  max: Signal<f64>,
  min: Signal<f64>,
  inverted: Signal<bool>,
  orientation: Signal<Orientation>,
  direction: Signal<Direction>,

  on_slide_start: Callback<f64>,
  on_slide_move: Callback<f64>,
  on_slide_end: Callback<()>,
  on_home_key_down: Callback<KeyboardEvent>,
  on_end_key_down: Callback<KeyboardEvent>,
  on_step_key_down: Callback<Step>,

  #[prop(attrs)] attrs: Attributes,
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let dom_rect = StoredValue::<Option<DomRect>>::new(None);

  provide_context(SliderImplContextValue { dom_rect });

  let children = StoredValue::new(children);

  move || {
    let attrs = attrs.clone();

    match orientation.get() {
      Orientation::Horizontal => view! {
          <SliderHorizontal
              max=max
              min=min
              inverted=inverted
              direction=direction
              node_ref=node_ref
          >
              <SliderImpl
                  max=max
                  min=min
                  inverted=inverted
                  orientation=orientation
                  direction=direction
                  on_slide_start=on_slide_start
                  on_slide_move=on_slide_move
                  on_slide_end=on_slide_end
                  on_home_key_down=on_home_key_down
                  on_end_key_down=on_end_key_down
                  on_step_key_down=on_step_key_down
                  node_ref=node_ref
                  attrs=attrs
                  as_child=as_child
              >
                  {children.with_value(|children| children())}
              </SliderImpl>
          </SliderHorizontal>
      },
      Orientation::Vertical => view! {
          <SliderVertical
              max=max
              min=min
              inverted=inverted
              direction=direction
              node_ref=node_ref
          >
              <SliderImpl
                  max=max
                  min=min
                  inverted=inverted
                  orientation=orientation
                  direction=direction
                  on_slide_start=on_slide_start
                  on_slide_move=on_slide_move
                  on_slide_end=on_slide_end
                  on_home_key_down=on_home_key_down
                  on_end_key_down=on_end_key_down
                  on_step_key_down=on_step_key_down
                  node_ref=node_ref
                  attrs=attrs
                  as_child=as_child
              >
                  {children.with_value(|children| children())}
              </SliderImpl>
          </SliderVertical>
      },
    }
  }
}

#[derive(Clone)]
struct SliderOrientationImplContextValue {
  pointer_value: Callback<i32, f64>,
  slide_direction: Signal<SlideDirection>,
}

#[component]
fn SliderHorizontal(
  max: Signal<f64>,
  min: Signal<f64>,
  inverted: Signal<bool>,
  direction: Signal<Direction>,
  node_ref: NodeRef<AnyElement>,
  children: Children,
) -> impl IntoView {
  let SliderImplContextValue { dom_rect } =
    use_context().expect("SliderImpl must be used in a Slider component");

  let is_left_to_right = Signal::derive(move || direction.get() == Direction::LeftToRight);
  let is_sliding_from_left = Signal::derive(move || {
    (is_left_to_right.get() && !inverted.get()) || (!is_left_to_right.get() && inverted.get())
  });

  let pointer_value = Callback::new(move |pointer: i32| {
    let rect = dom_rect
      .get_value()
      .unwrap_or(node_ref.get().unwrap().get_bounding_client_rect());

    let input = (0., rect.width());
    let output = if is_sliding_from_left.get() {
      (min.get(), max.get())
    } else {
      (max.get(), min.get())
    };
    let value = linear_scale(input, output);

    dom_rect.set_value(Some(rect.clone()));

    value(pointer as f64 - rect.left())
  });

  let slide_direction = Signal::derive(move || {
    if is_sliding_from_left.get() {
      SlideDirection::FromLeft
    } else {
      SlideDirection::FromRight
    }
  });

  provide_context(OrientationContextValue {
    start_edge: Signal::derive(move || {
      if is_sliding_from_left.get() {
        Side::Left
      } else {
        Side::Right
      }
    }),
    end_edge: Signal::derive(move || {
      if is_sliding_from_left.get() {
        Side::Right
      } else {
        Side::Left
      }
    }),
    direction: Signal::derive(move || {
      if is_sliding_from_left.get() {
        OrientationDirection::Forward
      } else {
        OrientationDirection::Backward
      }
    }),
    size: Signal::derive(|| Size::Width),
  });

  provide_context(SliderOrientationImplContextValue {
    pointer_value,
    slide_direction,
  });

  view! {
      <>{children()}</>
  }
}

#[component]
fn SliderVertical(
  max: Signal<f64>,
  min: Signal<f64>,
  inverted: Signal<bool>,
  direction: Signal<Direction>,
  node_ref: NodeRef<AnyElement>,
  children: Children,
) -> impl IntoView {
  let SliderImplContextValue { dom_rect } =
    use_context().expect("SliderImpl must be used in a Slider component");

  let is_sliding_from_bottom = Signal::derive(move || !inverted.get());

  let pointer_value = Callback::new(move |pointer: i32| {
    let rect = dom_rect
      .get_value()
      .unwrap_or(node_ref.get().unwrap().get_bounding_client_rect());

    let input = (0., rect.height());
    let output = if is_sliding_from_bottom.get() {
      (max.get(), min.get())
    } else {
      (min.get(), max.get())
    };
    let value = linear_scale(input, output);

    dom_rect.set_value(Some(rect.clone()));

    value(pointer as f64 - rect.top())
  });

  let slide_direction = Signal::derive(move || {
    if is_sliding_from_bottom.get() {
      SlideDirection::FromBottom
    } else {
      SlideDirection::FromTop
    }
  });

  provide_context(OrientationContextValue {
    start_edge: Signal::derive(move || {
      if is_sliding_from_bottom.get() {
        Side::Bottom
      } else {
        Side::Top
      }
    }),
    end_edge: Signal::derive(move || {
      if is_sliding_from_bottom.get() {
        Side::Top
      } else {
        Side::Bottom
      }
    }),
    direction: Signal::derive(move || {
      if is_sliding_from_bottom.get() {
        OrientationDirection::Forward
      } else {
        OrientationDirection::Backward
      }
    }),
    size: Signal::derive(|| Size::Height),
  });

  provide_context(SliderOrientationImplContextValue {
    pointer_value,
    slide_direction,
  });

  children().into_view()
}

#[component]
fn SliderImpl(
  max: Signal<f64>,
  min: Signal<f64>,
  inverted: Signal<bool>,
  orientation: Signal<Orientation>,
  direction: Signal<Direction>,

  on_slide_start: Callback<f64>,
  on_slide_move: Callback<f64>,
  on_slide_end: Callback<()>,
  on_home_key_down: Callback<KeyboardEvent>,
  on_end_key_down: Callback<KeyboardEvent>,
  on_step_key_down: Callback<Step>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let SliderImplContextValue { dom_rect } =
    use_context().expect("SliderImpl must be used in a Slider component");

  let SliderOrientationImplContextValue {
    pointer_value,
    slide_direction,
  } = use_context()
    .expect("SliderImpl must be used in either a SliderHorizontal or SliderVertical component");

  let context =
    use_context::<SliderContextValue>().expect("Slider must be used in a SliderRoot component");

  Effect::new(move |_| {
    if let Some(node) = node_ref.get() {
      _ = node.style(
        "--primitive-slider-thumb-transform",
        if orientation.get() == Orientation::Vertical {
          "translateY(50%)"
        } else {
          "translateX(-50%)"
        },
      );
    }
  });

  view! {
    <Primitive
      {..attrs}
      attr:data-orientation=move || orientation.get().to_string()
      attr:dir=move || (orientation.get() == Orientation::Horizontal).then_some(direction.get().to_string())
      element=html::span
      on:keydown=move |ev: KeyboardEvent| {
        if ev.key() == "Home" {
            on_home_key_down.call(ev.clone());
        } else if ev.key() == "End" {
            on_end_key_down.call(ev.clone());
        } else if ["PageUp", "PageDown", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].contains(&ev.key().as_ref()) {
        let is_back_key = match slide_direction.get() {
            SlideDirection::FromLeft => ["Home", "PageDown", "ArrowDown", "ArrowLeft"].contains(&ev.key().as_ref()),
            SlideDirection::FromRight => ["Home", "PageDown", "ArrowDown", "ArrowRight"].contains(&ev.key().as_ref()),
            SlideDirection::FromTop => ["Home", "PageDown", "ArrowDown", "ArrowLeft"].contains(&ev.key().as_ref()),
            SlideDirection::FromBottom => ["Home", "PageDown", "ArrowUp", "ArrowLeft"].contains(&ev.key().as_ref()),
        };

        on_step_key_down.call(Step {
            event: ev.clone(),
            direction: if is_back_key {
                OrientationDirection::Backward
            } else {
                OrientationDirection::Forward
            }
        });
        } else {
            return;
        }

        ev.prevent_default();
      }
      on:pointerdown=move |ev: PointerEvent| {
        let Some(target) = ev.target() else {
            return;
        };

        let Some(target_el) = target.dyn_ref::<web_sys::HtmlElement>() else {
            return;
        };

        _ = target_el.set_pointer_capture(ev.pointer_id());
        ev.prevent_default();

        if context.thumbs.get_value().iter().any(|el| {
          let el: &web_sys::HtmlElement = el;

          el == target_el
        }) {
          _ = target_el.focus();
        }

        on_slide_start.call(pointer_value.call(ev.client_x()));
      }
      on:pointermove=move |ev: PointerEvent| {
        let Some(target) = ev.target() else {
          return;
        };

        let Some(target_el) = target.dyn_ref::<web_sys::HtmlElement>() else {
          return;
        };

        if target_el.has_pointer_capture(ev.pointer_id()) {
            on_slide_move.call(pointer_value.call(if orientation.get() == Orientation::Horizontal { ev.client_x() } else { ev.client_y() }));
        }
      }
      on:pointerup=move |ev: PointerEvent| {
        let Some(target) = ev.target() else {
          return;
        };

        let Some(target_el) = target.dyn_ref::<web_sys::HtmlElement>() else {
          return;
        };

        if target_el.has_pointer_capture(ev.pointer_id()) {
          _ = target_el.release_pointer_capture(ev.pointer_id());

          dom_rect.set_value(None);

          on_slide_end.call(());
        }
      }
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn SliderTrack(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let SliderContextValue {
    disabled,
    orientation,
    ..
  } = use_context().expect("SliderTrack must be used in a SliderRoot component");

  view! {
    <Primitive
      {..attrs}
      attr:data-disabled=move || disabled.get().then_some("")
      attr:data-orientation=move || orientation.get().to_string()
      element=html::span
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn SliderRange(
  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  let context = use_context::<SliderContextValue>()
    .expect("SliderRange must be used in a SliderRoot component");
  let orientation = use_context::<OrientationContextValue>()
    .expect("SliderRange must be used in a SliderRoot component");

  let value_count = Signal::derive(move || context.values.get().len());
  let percentages = Signal::derive(move || {
    context
      .values
      .get()
      .iter()
      .map(|value| convert_value_to_percentage(*value, context.min.get(), context.max.get()))
      .collect::<Vec<_>>()
  });

  let offset_start = Signal::derive(move || {
    if value_count.get() > 1 {
      percentages
        .get()
        .iter()
        .fold(f64::INFINITY, |min, &x| min.min(x))
    } else {
      0.0f64
    }
  });
  let offset_end = Signal::derive(move || {
    100.0f64
      - percentages
        .get()
        .iter()
        .fold(f64::NEG_INFINITY, |max, &x| max.max(x))
  });

  Effect::new(move |_| {
    if let Some(node) = node_ref.get() {
      _ = node
        .style(
          orientation.start_edge.get().to_string().to_lowercase(),
          format!("{}%", offset_start.get()),
        )
        .style(
          orientation.end_edge.get().to_string().to_lowercase(),
          format!("{}%", offset_end.get()),
        );
    }
  });

  view! {
    <Primitive
      {..attrs}
      attr:data-disabled=move || context.disabled.get().then_some("")
      attr:data-orientation=move || context.orientation.get().to_string()
      element=html::span
      node_ref=node_ref
      as_child=as_child
    >
      {children()}
    </Primitive>
  }
}

#[component]
pub fn SliderThumb(
  #[prop(optional, into)] name: MaybeProp<String>,

  #[prop(optional)] node_ref: NodeRef<AnyElement>,
  #[prop(attrs)] attrs: Attributes,
  children: ChildrenFn,

  #[prop(optional, into)] as_child: MaybeProp<bool>,
) -> impl IntoView {
  use_collection_item_ref::<html::AnyElement, SliderCollectionItem>(node_ref, SliderCollectionItem);
  let get_items = use_collection_context::<SliderCollectionItem, AnyElement>();

  let context = use_context::<SliderContextValue>()
    .expect("SliderThumb must be used in a SliderRoot component");
  let orientation = use_context::<OrientationContextValue>()
    .expect("SliderThumb must be used in a SliderRoot component");

  let (is_form_control, set_is_form_control) = create_signal(true);

  let size = use_element_size(node_ref);

  let index = Signal::derive(move || {
    let node = node_ref.get()?;
    let items = get_items.get();

    let index = items.iter().position(|item| {
      let Some(item) = item.0.get() else {
        return false;
      };

      let item_el: &web_sys::Element = &item;
      let node_el: &web_sys::Element = &node;

      item_el == node_el
    })?;

    Some(index)
  });

  let value = Signal::derive(move || {
    let result = *context.values.get().get(index.get()?)?;
    Some(result)
  });

  let percent = Signal::derive(move || {
    value
      .get()
      .map(|value| convert_value_to_percentage(value, context.min.get(), context.max.get()))
      .unwrap_or(0.)
  });

  let label = Signal::derive(move || {
    index
      .get()
      .and_then(|index| get_label(index, context.values.get().len()))
  });

  let orientation_size = Signal::derive(move || match orientation.size.get() {
    Size::Width => size.width.get(),
    Size::Height => size.height.get(),
  });

  let thumbs_in_bound_offset = Signal::derive(move || {
    get_thumb_in_bounds_offset(
      orientation_size.get(),
      percent.get(),
      match orientation.direction.get() {
        OrientationDirection::Backward => -1.0f64,
        OrientationDirection::Forward => 1.0f64,
      },
    )
  });

  Effect::new(move |_| {
    set_is_form_control.set(if let Some(foo) = node_ref.get() {
      foo.closest("form").ok().flatten().is_some()
    } else {
      true
    });
  });

  Effect::new(move |_| {
    let Some(node) = node_ref.get() else {
      return;
    };

    context.thumbs.update_value(|thumbs| {
      thumbs.push(node.clone());
    });

    on_cleanup(move || {
      // let Some(node) = node_ref.get() else {
      //   return;
      // };

      _ = context.thumbs.try_update_value(|thumbs| {
        if let Some(position) = thumbs.iter().position(|thumb| {
          let thumb_el: &web_sys::Element = thumb;
          let node_el: &web_sys::Element = &node.clone();

          thumb_el == node_el
        }) {
          _ = thumbs.remove(position);
        }
      });
    });
  });

  Effect::new(move |_| {
    let Some(node) = node_ref.get() else {
      return;
    };

    let node = node.on(focus, move |_| {
      context.value_index_to_change.set_value(index.get());
    });

    if value.get().is_none() {
      _ = node.clone().style("display", "none");
    }
  });

  let span_ref = NodeRef::<Span>::new();

  Effect::new(move |_| {
    if let Some(node) = span_ref.get() {
      _ = node.style(
        orientation.start_edge.get().to_string().to_lowercase(),
        format!(
          "calc({}% + {}px)",
          percent.get(),
          thumbs_in_bound_offset.get()
        ),
      );
    }
  });

  view! {
    <span style:transform="var(--primitive-slider-thumb-transform)" style:position="absolute" node_ref=span_ref>
      <Primitive
        {..attrs}
        attr:role="slider"
        attr:aria-label=name.clone()
        attr:aria-valuemin=context.min
        attr:aria-valuenow=move || value.get().unwrap_or_default()
        attr:aria-valuemax=context.max
        attr:aria-orientation=move || context.orientation.get().to_string()
        attr:data-orientation=move || context.orientation.get().to_string()
        attr:data-disabled=move || context.disabled.get().then_some("")
        attr:tabindex=move || (!context.disabled.get()).then_some(0)
        element=html::span
        node_ref=node_ref
        as_child=as_child
      >
        {children()}
      </Primitive>

     <Show when=move || is_form_control.get()>
       <BubbleInput
           name=name.clone()
         value=Signal::derive(move || value.get().unwrap_or_default())
       />
     </Show>
    </span>
  }
}

#[component]
fn BubbleInput(
  #[prop(optional, into)] name: MaybeProp<String>,
  value: Signal<f64>,
) -> impl IntoView {
  let SliderContextValue { values, .. } =
    use_context().expect("SliderThumb must be used in a SliderRoot component");

  let node_ref = NodeRef::<Input>::new();
  let prev_value = create_previous(Signal::derive(move || value.get()));

  Effect::new(move |_| {
    (|| {
      let input = node_ref.get()?;
      let input_el = window().get("HTMLInputElement")?;
      let input_proto = Reflect::get(&input_el, &JsString::from("prototype"))
        .ok()?
        .dyn_into::<Object>()
        .ok()?;

      let input_descriptor_set = Reflect::get(
        &Object::get_own_property_descriptor(&input_proto, &JsString::from("value")),
        &JsString::from("set"),
      )
      .ok()?
      .dyn_into::<Function>()
      .ok()?;

      if prev_value.get() != value.get() {
        let mut ev_options = EventInit::new();
        ev_options.bubbles(true);

        let ev = Event::new_with_event_init_dict("input", &ev_options).ok()?;

        _ = Reflect::apply(
          &input_descriptor_set,
          &input,
          &Array::from_iter([JsValue::from_f64(value.get())]),
        );

        _ = input.dispatch_event(&ev);
      }

      Some(())
    })();
  });

  Effect::new(move |_| {
    if let Some(node) = node_ref.get() {
      node.set_default_value(&value.get().to_string());
    }
  });

  view! {
    <input
      aria-hidden
      name=Signal::derive(move || name.get().map(|name| format!("{}{}", name, if values.get().len() > 1 { "[]" } else { "" }))).into_attribute()
      value=value.into_attribute()
      node_ref=node_ref
      style:display="none"
    />
  }
}

fn get_label(index: usize, total_values: usize) -> Option<String> {
  match total_values {
    n if n > 2 => Some(format!("Value {} of {}", index + 1, total_values)),
    2 => ["Minimum", "Maximum"]
      .get(index)
      .map(|label| label.to_string()),
    _ => None,
  }
}

fn convert_value_to_percentage(value: f64, min: f64, max: f64) -> f64 {
  let max_steps = max - min;
  let percent_per_step = 100. / max_steps;
  let percentage = percent_per_step * (value - min);

  percentage.clamp(0., 100.)
}

fn get_next_sorted_values(prev_values: &Vec<f64>, next_value: f64, at_index: usize) -> Vec<f64> {
  let mut next_values = prev_values.clone();
  if let Some(next_values) = next_values.get_mut(at_index) {
    *next_values = next_value;
  };

  next_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
  next_values
}

fn find_closest_index(values: &[f64], next_value: f64) -> Option<usize> {
  values
    .iter()
    .enumerate()
    .min_by_key(|(_, &value)| (next_value - value).abs() as i32)
    .map(|(index, _)| index)
}

fn has_min_steps_between_values(values: &[f64], min_steps_between_values: f64) -> bool {
  if min_steps_between_values <= 0. {
    return true;
  }

  values
    .windows(2)
    .map(|pair| pair[1] - pair[0])
    .fold(None, |min: Option<f64>, current| match min {
      Some(min_val) => Some(min_val.min(current)),
      None => Some(current),
    })
    .map(|steps_between_values| steps_between_values >= min_steps_between_values)
    .unwrap_or(false)
}

fn get_decimal_count(value: f64) -> usize {
  value.to_string().split('.').nth(1).unwrap_or("").len()
}

fn round_value(value: f64, decimal_count: u32) -> f64 {
  let rounder = 10_f64.powi(decimal_count as i32);
  (value * rounder).round() / rounder
}

fn get_thumb_in_bounds_offset(width: f64, left: f64, direction: f64) -> f64 {
  let half_width = width / 2.0;
  let half_percent = 50.0;
  let offset = linear_scale((0.0, half_percent), (0.0, half_width));

  (half_width - offset(left) * direction) * direction
}
