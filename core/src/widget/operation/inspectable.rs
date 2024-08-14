#![allow(missing_docs)]
use std::collections::VecDeque;

use crate::widget;
use crate::widget::operation::Outcome;
use crate::Rectangle;
use crate::{widget::Operation, Length, Padding, Size};
// use crate::alignment;

use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Default)]
pub struct Properties {
    pub name: String,
    // pub size: Option<Size<Length>>,
    // pub max_size: Option<Size<f32>>,
    // pub padding: Option<Padding>,
    // horizontal_alignment: Option<alignment::Horizontal>,
    // vertical_alignment: Option<alignment::Vertical>,
    // clip: Option<bool>,
}

pub trait Inspectable {
    fn properties(&self) -> Properties;
}

#[derive(Debug, Clone)]
pub struct Map {
    root: Option<widget::Id>,
    elements: FxHashMap<widget::Id, Element>,
}

#[derive(Debug, Default, Clone)]
struct Element {
    parent: Option<widget::Id>,
    bounds: Rectangle,
    properties: Properties,
    children: Vec<widget::Id>,
}

pub fn map() -> impl Operation<Map> {
    struct Container {
        id: widget::Id,
        children: Vec<widget::Id>,
    }

    #[derive(Default)]
    struct InspectableMap {
        root: Option<widget::Id>,
        parent: VecDeque<Container>,
        elements: FxHashMap<widget::Id, Element>,
    }

    impl Operation<Map> for InspectableMap {
        fn inspectable(
            &mut self,
            id: Option<&widget::Id>,
            bounds: Rectangle,
            state: &mut dyn Inspectable,
        ) {
            let id = id.cloned().unwrap_or(widget::Id::unique());

            if let Some(parent) = self.parent.back_mut() {
                parent.children.push(id.clone());
            }

            let properties = state.properties();
            println!("Inspecting: {:?}({id:?})", properties.name);

            let element = Element {
                parent: self.parent.back_mut().map(|p| p.id.clone()),
                bounds,
                properties,
                children: vec![],
            };

            let _ = self.elements.insert(id, element);
        }

        fn container(
            &mut self,
            id: Option<&widget::Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Map>),
        ) {
            let id = id.cloned().unwrap_or(widget::Id::unique());
            let container = Container {
                id: id.clone(),
                children: vec![],
            };

            self.parent.push_back(container);
            operate_on_children(self);
            let children =
                self.parent.pop_back().map(|c| c.children).unwrap_or(vec![]);
            let _ = self.elements.entry(id).and_modify(|element| {
                element.children = children;
            });
        }

        fn finish(&self) -> Outcome<Map> {
            let map = Map {
                root: self.root.clone(),
                elements: self.elements.clone(),
            };

            Outcome::Some(map)
        }
    }

    InspectableMap::default()
}
