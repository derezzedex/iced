#![allow(missing_docs)]
use crate::widget;
use crate::widget::operation::Outcome;
use crate::widget::Operation;
use crate::Rectangle;
use crate::Size;

use core::panic::Location;

use rustc_hash::FxHashMap;

pub use serde::{Deserialize, Serialize};
pub use serde_json::to_string_pretty;

#[derive(Debug, Clone)]
pub struct Specific(serde_json::Value);

impl Specific {
    pub fn from_str(content: &str) -> Self {
        Self(
            serde_json::from_str(content).expect("failed to serialize content"),
        )
    }

    pub fn serialize<T: serde::Serialize>(content: T) -> Self {
        Self(
            serde_json::to_value(content).expect("failed to serialize content"),
        )
    }

    pub fn try_serialize<T: serde::Serialize>(content: T) -> Option<Self> {
        serde_json::to_value(content).ok().map(Self)
    }

    pub fn deserialize<T>(self) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_value(self.0).expect("failed to deserialize content")
    }

    pub fn try_deserialize<T>(self) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_value(self.0).ok()
    }

    pub fn find_and_get<T>(&self) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.0
            .as_object()
            .map(|fields| {
                fields
                    .values()
                    .find_map(|v| serde_json::from_value(v.clone()).ok())
            })
            .flatten()
    }

    pub fn fields(&self) -> &serde_json::Map<String, serde_json::Value> {
        self.0.as_object().expect("failed to obtain json object")
    }

    pub fn to_string_pretty(&self) -> Option<String> {
        serde_json::to_string_pretty(&self.0).ok()
    }
}
#[derive(Debug, Clone)]
pub struct Properties {
    pub id: widget::Id,
    pub name: String,
    pub location: Location<'static>,
    pub specific: Specific,
    pub messages: Specific,
}

impl PartialEq for Properties {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl Inspectable for Properties {
    fn edit(&mut self, specific: String) {
        self.specific = Specific::from_str(&specific);
    }

    fn properties(&self) -> &Properties {
        &self
    }
}

pub trait Inspectable {
    fn edit(&mut self, specific: String);
    fn properties(&self) -> &Properties;
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

#[derive(Debug, Clone)]
pub struct Element {
    // TODO: parent: Option<widget::Id>,
    pub bounds: Rectangle,
    pub properties: Properties,
    // children: Vec<widget::Id>,
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
        // parent: VecDeque<Container>,
        elements: FxHashMap<widget::Id, Element>,
    }

    impl Operation<Map> for InspectableMap {
        fn inspectable(
            &mut self,
            state: &mut dyn Inspectable,
            _id: Option<&widget::Id>,
            bounds: Rectangle,
        ) {
            let properties = state.properties();
            let id = properties.id.clone();

            // if let Some(parent) = self.parent.back_mut() {
            //     parent.children.push(id.clone());
            // }

            let element = Element {
                // TODO: parent: self.parent.back_mut().map(|p| p.id.clone()),
                bounds,
                properties: properties.clone(),
                // children: vec![],
            };

            let _ = self.elements.insert(id, element);
        }

        fn container(
            &mut self,
            _id: Option<&widget::Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Map>),
        ) {
            // let id = id.cloned().unwrap_or(widget::Id::unique());
            // let container = Container { children: vec![] };

            // self.parent.push_back(container);
            operate_on_children(self);
            // let children =
            //     self.parent.pop_back().map(|c| c.children).unwrap_or(vec![]);
            // let _ = self.elements.entry(id).and_modify(|element| {
            //     element.children = children;
            // });
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

pub fn edit(target: widget::Id, specific: String) -> impl Operation {
    struct Focus {
        target: widget::Id,
        specific: String,
    }

    impl<T> Operation<T> for Focus {
        fn inspectable(
            &mut self,
            state: &mut dyn Inspectable,
            _id: Option<&widget::Id>,
            _bounds: Rectangle,
        ) {
            if state.properties().id == self.target {
                state.edit(self.specific.clone());
            }
        }

        fn container(
            &mut self,
            _id: Option<&widget::Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self);
        }
    }

    Focus { target, specific }
}
