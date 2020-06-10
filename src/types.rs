use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[readonly::make]
#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    /// Describes the nature of an entity's content based on the current
    /// representation. Possible values are implementation-dependent and should
    /// be documented. MUST be an array of strings. Optional.
    #[serde(default)]
    pub class: Vec<String>,
    /// A set of key-value pairs that describe the state of an entity. In JSON
    /// Siren, this is an object such as { "name": "Kevin", "age": 30 }.
    /// Optional.
    #[serde(default)]
    pub properties: serde_json::Value,
    /// A collection of related sub-entities. If a sub-entity contains an href
    /// value, it should be treated as an embedded link. Clients may choose to
    /// optimistically load embedded links. If no href value exists, the
    /// sub-entity is an embedded entity representation that contains all the
    /// characteristics of a typical entity. One difference is that a sub-entity
    /// MUST contain a rel attribute to describe its relationship to the parent
    /// entity.
    // In JSON Siren, this is represented as an array. Optional.
    #[serde(default)]
    pub entities: Vec<SubEntity>,
    /// A collection of items that describe navigational links, distinct from
    /// entity relationships. Link items should contain a rel attribute to
    /// describe the relationship and an href attribute to point to the target
    /// URI. Entities should include a link rel to self. In JSON Siren, this is
    /// represented as "links": `[{ "rel": ["self"], "href": "http://api.x.io/orders/1234" }]`
    /// Optional.
    #[serde(default)]
    pub links: Vec<NavigationalLink>,
    /// A collection of action objects, represented in JSON Siren as an array
    /// such as { "actions": [{ ... }] }. See Actions. Optional
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Descriptive text about the entity. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            class: Vec::default(),
            properties: serde_json::Value::Object(serde_json::Map::default()),
            entities: Vec::default(),
            links: Vec::default(),
            actions: Vec::default(),
            title: None,
        }
    }
}

#[derive(Debug)]
pub enum EntityBuilderError {
    /// Whatever you passed, it doesn't serialize to a JSON object
    NotAnObject,
    Serde(serde_json::Error),
}

impl fmt::Display for EntityBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self {
            EntityBuilderError::NotAnObject => "does not serialize to an object",
            EntityBuilderError::Serde(_) => "serialization failure",
        };

        write!(f, "{}", desc)
    }
}

impl std::error::Error for EntityBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EntityBuilderError::NotAnObject => None,
            EntityBuilderError::Serde(inner) => Some(inner),
        }
    }
}

impl From<serde_json::Error> for EntityBuilderError {
    fn from(serde_error: serde_json::Error) -> Self {
        EntityBuilderError::Serde(serde_error)
    }
}

impl Entity {
    pub fn with_properties<S: Serialize>(
        self,
        serializable: S,
    ) -> Result<Self, EntityBuilderError> {
        let value = serde_json::to_value(serializable)?;

        match &value {
            serde_json::Value::Object(_) => Ok(Entity {
                properties: value,
                ..self
            }),
            _ => Err(EntityBuilderError::NotAnObject),
        }
    }

    pub fn with_class_member(mut self, class_member: impl Into<String>) -> Self {
        self.class.push(class_member.into());

        self
    }

    pub fn with_link(mut self, link: NavigationalLink) -> Self {
        self.links.push(link);

        self
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.actions.push(action);

        self
    }

    pub fn push_sub_entity(&mut self, sub_entity: SubEntity) {
        self.entities.push(sub_entity);
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SubEntity {
    Link {
        #[serde(flatten)]
        inner: EntityLink,
    },
    Embedded {
        #[serde(flatten)]
        inner: Entity,
        /// Defines the relationship of the sub-entity to its parent, per Web
        /// Linking (RFC5988) and Link Relations. MUST be a non-empty array of
        /// strings. Required.
        #[serde(default)]
        rel: Vec<String>,
    },
}

impl SubEntity {
    pub fn from_link(entity_link: EntityLink) -> Self {
        SubEntity::Link { inner: entity_link }
    }

    pub fn from_entity(entity: Entity, rels: &[impl Into<String> + Clone]) -> Self {
        SubEntity::Embedded {
            inner: entity,
            rel: rels.iter().map(|rel| rel.clone().into()).collect(),
        }
    }
}

#[readonly::make]
#[derive(Debug, Serialize, Deserialize)]
pub struct EntityLink {
    /// Describes the nature of an entity's content based on the current
    /// representation. Possible values are implementation-dependent and
    /// should be documented. MUST be an array of strings. Optional.
    #[serde(default)]
    pub class: Vec<String>,
    /// Descriptive text about the entity. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Defines the relationship of the sub-entity to its parent, per Web
    /// Linking (RFC5988) and Link Relations. MUST be a non-empty array of
    /// strings. Required.
    #[serde(default)]
    pub rel: Vec<String>,
    /// The URI of the linked sub-entity. Required.
    pub href: String,
    /// Defines media type of the linked sub-entity, per Web Linking
    /// (RFC5988). Optional.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<String>,
}

#[readonly::make]
#[derive(Debug, Serialize, Deserialize)]
pub struct NavigationalLink {
    /// Defines the relationship of the link to its entity, per Web Linking
    /// (RFC5988) and Link Relations. MUST be an array of strings. Required.
    pub rel: Vec<String>,
    /// Describes aspects of the link based on the current representation.
    /// Possible values are implementation-dependent and should be documented.
    /// MUST be an array of strings. Optional.
    #[serde(default)]
    pub class: Vec<String>,
    /// The URI of the linked resource. Required.
    pub href: String,
    /// Text describing the nature of a link. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Defines media type of the linked resource, per Web Linking (RFC5988).
    /// Optional.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<String>,
}

impl NavigationalLink {
    pub fn new(rels: &[impl Into<String> + Clone], href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            rel: rels.iter().map(|rel| rel.clone().into()).collect(),
            class: Vec::new(),
            title: None,
            _type: None,
        }
    }

    pub fn with_class_member(mut self, class_member: impl Into<String>) -> Self {
        self.class.push(class_member.into());

        self
    }

    pub fn with_type(mut self, _type: impl Into<String>) -> Self {
        self._type = Some(_type.into());

        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());

        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    /// A string that identifies the action to be performed. Action names MUST
    /// be unique within the set of actions for an entity. The behaviour of
    /// clients when parsing a Siren document that violates this constraint is
    /// undefined. Required.
    pub name: String,
    /// Describes the nature of an action based on the current representation.
    /// Possible values are implementation-dependent and should be documented.
    /// MUST be an array of strings. Optional.
    #[serde(default)]
    pub class: Vec<String>,
    /// An enumerated attribute mapping to a protocol method. For HTTP, these
    /// values may be GET, PUT, POST, DELETE, or PATCH. As new methods are
    /// introduced, this list can be extended. If this attribute is omitted, GET
    /// should be assumed. Optional.
    #[serde(with = "crate::http_serde::option_method")]
    pub method: Option<http::Method>,
    /// The URI of the action. Required.
    pub href: String,
    /// Descriptive text about the action. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The encoding type for the request. When omitted and the fields attribute
    /// exists, the default value is application/x-www-form-urlencoded.
    /// Optional.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<String>,
    /// A collection of fields, expressed as an array of objects in JSON Siren
    /// such as { "fields" : [{ ... }] }. See Fields. Optional.
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    /// A name describing the control. Field names MUST be unique within the set
    /// of fields for an action. The behaviour of clients when parsing a Siren
    /// document that violates this constraint is undefined. Required.
    pub name: String,
    /// Describes aspects of the field based on the current representation.
    /// Possible values are implementation-dependent and should be documented.
    /// MUST be an array of strings. Optional.
    #[serde(default)]
    pub class: Vec<String>,
    /// The input type of the field. This may include any of the following input
    /// types specified in HTML5:
    /// hidden, text, search, tel, url, email, password, datetime, date, month,
    /// week, time, datetime-local, number, range, color, checkbox, radio,
    /// file
    //
    /// When missing, the default value is text. Serialization of these fields
    /// will depend on the value of the action's type attribute. See type
    /// under Actions, above. Optional.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<String>,
    /// A value assigned to the field. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Textual annotation of a field. Clients may use this as a label.
    /// Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
