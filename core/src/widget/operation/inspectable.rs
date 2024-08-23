#![allow(missing_docs)]
use std::collections::VecDeque;

use crate::widget;
use crate::widget::operation::Outcome;
use crate::widget::Operation;
use crate::Rectangle;
use crate::Size;

use rustc_hash::FxHashMap;

use core::panic::Location;

#[derive(Debug, PartialEq, Clone)]
pub struct Properties {
    pub name: String,
    pub location: Location<'static>,
}

pub trait Inspectable {
    fn properties(&self) -> Properties;
}

#[derive(Debug, Clone)]
pub struct Map {
    // TODO: root: Option<widget::Id>,
    elements: FxHashMap<widget::Id, Element>,
}

impl Map {
    pub fn widgets(&self) -> impl Iterator<Item = &Element> {
        self.elements.values()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Element {
    // TODO: parent: Option<widget::Id>,
    pub bounds: Rectangle,
    pub properties: Properties,
    children: Vec<widget::Id>,
}

impl Element {
    pub fn size(&self) -> Size {
        self.bounds.size()
    }
}

pub fn map() -> impl Operation<Map> {
    #[derive(Debug)]
    struct Container {
        // TODO: id: widget::Id,
        children: Vec<widget::Id>,
    }

    #[derive(Default)]
    struct InspectableMap {
        // TODO: root: Option<widget::Id>,
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

            let element = Element {
                // TODO: parent: self.parent.back_mut().map(|p| p.id.clone()),
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
            let container = Container { children: vec![] };

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
                // TODO: root: self.root.clone(),
                elements: self.elements.clone(),
            };

            Outcome::Some(map)
        }
    }

    InspectableMap::default()
}
