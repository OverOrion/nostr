//! NIP58
//!
//! <https://github.com/nostr-protocol/nips/blob/master/58.md>

use crate::{
    event::{builder::Error as BuilderError, tag::UncheckedUrl},
    Event, EventBuilder, Keys, Kind, Tag,
};

#[derive(Debug, thiserror::Error)]
/// [`BadgeAward`] error
pub enum Error {
    /// Invalid kind
    #[error("invalid kind")]
    InvalidKind,
    /// Event builder Error
    #[error(transparent)]
    Event(#[from] crate::event::builder::Error),
}

/// Simple struct to hold `width` x `height.
pub struct ImageDimensions(u64, u64);

/// [`BadgeDefinition`] event builder
pub struct BadgeDefinitionBuilder {
    badge_id: String,
    name: Option<String>,
    image: Option<String>,
    image_dimensions: Option<ImageDimensions>,
    description: Option<String>,
    thumbs: Option<Vec<(String, Option<ImageDimensions>)>>,
}

impl BadgeDefinitionBuilder {
    /// New [`BadgeDefinitionBuilder`]
    pub fn new(badge_id: String) -> Self {
        Self {
            badge_id,
            name: None,
            image: None,
            image_dimensions: None,
            description: None,
            thumbs: None,
        }
    }

    /// Set name
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set image
    pub fn image(mut self, image: String) -> Self {
        self.image = Some(image);
        self
    }

    /// Set `[ImageDimensions]`
    pub fn image_dimensions(mut self, image_dimensions: ImageDimensions) -> Self {
        self.image_dimensions = Some(image_dimensions);
        self
    }

    /// Set description
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set thumbnails with their optional `[ImageDimensions]`
    pub fn thumbs(mut self, thumbs: Vec<(String, Option<ImageDimensions>)>) -> Self {
        self.thumbs = Some(thumbs);
        self
    }

    /// Build [`Event`]
    pub fn build(self, keys: &Keys) -> Result<BadgeDefinition, BuilderError> {
        let mut tags: Vec<Tag> = vec![];
        let badge_id = Tag::Identifier(self.badge_id);
        tags.push(badge_id);

        if let Some(name) = self.name {
            let name_tag = Tag::Name(name);
            tags.push(name_tag);
        };

        if let Some(image) = self.image {
            let image_tag = if let Some(width_height) = self.image_dimensions {
                let ImageDimensions(width, height) = width_height;
                Tag::Image(image, Some((width, height)))
            } else {
                Tag::Image(image, None)
            };
            tags.push(image_tag);
        }

        if let Some(description) = self.description {
            let description_tag = Tag::Description(description);
            tags.push(description_tag);
        }
        if let Some(thumbs) = self.thumbs {
            for thumb in thumbs {
                let thumb_url = thumb.0;
                let thumb_tag = if let Some(width_height) = thumb.1 {
                    let ImageDimensions(width, height) = width_height;
                    Tag::Thumb(thumb_url, Some((width, height)))
                } else {
                    Tag::Thumb(thumb_url, None)
                };
                tags.push(thumb_tag);
            }
        }

        let event_builder = EventBuilder::new(Kind::BadgeDefinition, String::new(), &tags);
        let event = event_builder.to_event(keys)?;
        Ok(BadgeDefinition(event))
    }
}

/// Badge definition event as specified in NIP-58
pub struct BadgeDefinition(Event);

/// Badge award event as specified in NIP-58
pub struct BadgeAward(Event);

impl BadgeAward {
    ///
    pub fn new(
        badge_definition: &Event,
        awarded_pub_keys: Vec<Tag>,
        keys: &Keys,
    ) -> Result<BadgeAward, Error> {
        match badge_definition.kind {
            Kind::BadgeDefinition => (),
            _ => return Err(Error::InvalidKind),
        };

        let awarded_pub_keys: Vec<Tag> = awarded_pub_keys
            .into_iter()
            .filter(|e| matches!(e, Tag::PubKey(..)))
            .collect();

        if awarded_pub_keys.is_empty() {
            return Err(Error::InvalidKind);
        }

        let mut tags = badge_definition.tags.clone();
        dbg!(tags.clone());
        tags.extend(awarded_pub_keys);

        let event_builder = EventBuilder::new(Kind::BadgeAward, String::new(), &tags);
        let event = event_builder.to_event(keys)?;

        Ok(BadgeAward(event))
    }
}

///  Profile Badges event as specified in NIP-58
pub struct ProfileBadgesEvent(Event);

/// [`ProfileBadgesEvent`] errors
#[derive(Debug, thiserror::Error)]
pub enum ProfileBadgesEventError {
    /// Invalid length
    #[error("invalid length")]
    InvalidLength,
    /// Invalid kind
    #[error("invalid kind")]
    InvalidKind,
    /// Mismatched badge definition or award
    #[error("mismatched badge definition/award")]
    MismatchedBadgeDefinitionOrAward,
    /// Event builder Error
    #[error(transparent)]
    EventBuilder(#[from] crate::event::builder::Error),
}

impl ProfileBadgesEvent {
    /// Helper function to filter events for a specific [`Kind`]
    pub(crate) fn filter_for_kind(events: Vec<Event>, kind_needed: &Kind) -> Vec<Event> {
        events
            .into_iter()
            .filter(|e| e.kind == *kind_needed)
            .collect()
    }
    fn extract_identifier(tags: Vec<Tag>) -> Option<Tag> {
        dbg!(tags.clone());
        tags.iter()
            .find(|tag| match tag {
                Tag::Identifier(_) => true,
                _ => false,
            })
            .cloned()
    }
    fn extract_relay_url(tags: Vec<Tags>) -> Option<UncheckedUrl> {
        tags.iter()
            .find(|tag| match tag {
                Tag::Event(_, UncheckedUrl, ..) => uncheckedurl,
                _ => None,
            })
            .cloned()
    }

    /// Create a new [`ProfileBadgesEvent`] from badge definition and awards events
    /// [`badge_definitions`] and [`badge_awards`] must be ordered, so on the same position they refer to the same badge
    pub fn new(
        badge_definitions: Vec<Event>,
        badge_awards: Vec<Event>,
        keys: &Keys,
    ) -> Result<ProfileBadgesEvent, ProfileBadgesEventError> {
        if badge_definitions.len() != badge_awards.len() {
            return Err(ProfileBadgesEventError::InvalidLength);
        }
        dbg!(badge_awards.clone());

        let mut badge_awards = ProfileBadgesEvent::filter_for_kind(badge_awards, &Kind::BadgeAward);
        if badge_awards.is_empty() {
            return Err(ProfileBadgesEventError::InvalidKind);
        }

        let mut badge_definitions =
            ProfileBadgesEvent::filter_for_kind(badge_definitions, &Kind::BadgeDefinition);
        if badge_definitions.is_empty() {
            return Err(ProfileBadgesEventError::InvalidKind);
        }

        // Add identifier `d` tag
        let id_tag = Tag::Identifier("profile_badges".to_owned());
        let mut tags: Vec<Tag> = vec![id_tag];

        let badge_definitions_identifiers: Vec<_> = badge_definitions
            .iter_mut()
            .map(|event| {
                let tags = core::mem::take(&mut event.tags);
                let id = Self::extract_identifier(tags.clone())
                    .expect("BadgeDefinitions events should have identifier tags")
                    .clone();
                (event, id)
            })
            .collect();

        let badge_awards_identifiers: Vec<_> = badge_awards
            .iter_mut()
            .map(|event| {
                let tags = core::mem::take(&mut event.tags);
                let id = Self::extract_identifier(tags.clone())
                    .expect("BadgeAward events should have identifier tags")
                    .clone();
                (event, id)
            })
            .collect();
        //dbg!(badge_awards_identifiers.());
        // This collection has been filtered for the needed tags
        let users_badges: Vec<(_, _)> = dbg!(core::iter::zip(
            badge_definitions_identifiers,
            badge_awards_identifiers
        ))
        .collect();
        dbg!(users_badges);
        //unimplemented!();
        for (badge_definition, badge_award) in users_badges {
            match (&badge_definition, &badge_award) {
                ((_, Tag::Identifier(identifier)), (_, Tag::Identifier(badge_id)))
                    if badge_id != identifier =>
                {
                    return Err(ProfileBadgesEventError::MismatchedBadgeDefinitionOrAward);
                }
                (
                    (badge_definition_event, Tag::Identifier(identifier)),
                    (badge_award_event, Tag::Identifier(badge_id)),
                ) if badge_id == identifier => {
                    let badge_definition_event_tag = Tag::Event(badge_definition_event.id, (), ());
                    tags.extend_from_slice(&[badge_definition_event, badge_award_event]);
                }
                _ => {}
            }
        }

        // Badge definitions and awards have been validated

        let event_builder = EventBuilder::new(Kind::ProfileBadges, String::new(), &tags);
        let event = event_builder.to_event(keys)?;

        Ok(ProfileBadgesEvent(event))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use secp256k1::XOnlyPublicKey;

    use super::*;
    use crate::prelude::tag;

    fn get_badge_with_id_only(id: String, keys: &Keys) -> BadgeDefinition {
        let builder = BadgeDefinitionBuilder::new(id);
        builder.build(keys).unwrap()
    }

    #[test]
    fn test_badge_definition_builder() {
        let example_event_json = r#"{"content":"","id":"378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7","sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8","created_at":1671739153,"pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","kind":30009,"tags":[["d","bravery"],["name","Medal of Bravery"],["image","https://nostr.academy/awards/bravery.png","1024x1024"],["description","Awarded to users demonstrating bravery"],["thumb","https://nostr.academy/awards/bravery_256x256.png","256x256"]]}"#;

        let example_event: Event = serde_json::from_str(example_event_json).unwrap();

        let mut builder = BadgeDefinitionBuilder::new("bravery".to_owned());
        let image_dimensions = ImageDimensions(1024, 1024);
        let thumb_size = ImageDimensions(256, 256);
        let thumbs = vec![(
            "https://nostr.academy/awards/bravery_256x256.png".to_owned(),
            Some(thumb_size),
        )];
        builder = builder
            .name("Medal of Bravery".to_owned())
            .description("Awarded to users demonstrating bravery".to_owned())
            .image("https://nostr.academy/awards/bravery.png".to_owned())
            .image_dimensions(image_dimensions)
            .thumbs(thumbs);

        let keys = Keys::generate();
        let badge_definition_event = builder.build(&keys).unwrap().0;

        assert_eq!(badge_definition_event.kind, Kind::BadgeDefinition);
        assert_eq!(badge_definition_event.tags, example_event.tags);
    }
    #[test]
    fn test_badge_award() {
        let example_event_json = r#"{ "content":"","id": "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7", "kind": 8, "pubkey": "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3", "sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8","created_at":1671739153,"tags": [ ["a","30009:79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3:bravery"],["p", "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3", "wss://relay"], ["p", "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3", "wss://relay"] ] }"#;
        let example_event: Event = serde_json::from_str(example_event_json).unwrap();

        let keys = Keys::generate();
        let pub_key = XOnlyPublicKey::from_str(
            "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3",
        )
        .unwrap();

        let relay_url = tag::UncheckedUrl::from_str("wss://relay").unwrap();
        let badge_definition = get_badge_with_id_only("bravery".to_owned(), &keys).0;

        let awarded_pub_keys = vec![
            Tag::PubKey(pub_key.clone(), Some(relay_url.clone())),
            Tag::PubKey(pub_key.clone(), Some(relay_url.clone())),
        ];
        let badge_award = BadgeAward::new(&badge_definition, awarded_pub_keys, &keys)
            .unwrap()
            .0;

        assert_eq!(badge_award.kind, Kind::BadgeAward);
        assert_eq!(badge_award.tags, example_event.tags);
    }

    #[test]
    fn test_profile_badges() {
        let example_event_json = r#"{ "content":"","id": "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7", "kind": 30008, "pubkey": "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3", "sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8","created_at":1671739153,"tags": [ ["d", "profile_badges"],["a", "30009:79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3:bravery"],["e", "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7", "wss://nostr.academy"],["a", "30009:79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3:honor"],["e", "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7", "wss://nostr.academy"]] }"#;
        let example_event: Event = serde_json::from_str(example_event_json).unwrap();

        let pub_key = XOnlyPublicKey::from_str(
            "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3",
        )
        .unwrap();
        let relay_url = tag::UncheckedUrl::from_str("wss://relay").unwrap();
        let keys = Keys::generate();

        let awarded_pub_keys = vec![
            Tag::PubKey(pub_key.clone(), Some(relay_url.clone())),
            Tag::PubKey(pub_key.clone(), Some(relay_url.clone())),
        ];
        let bravery_badge_event = get_badge_with_id_only("bravery".to_owned(), &keys).0;
        dbg!(bravery_badge_event.clone());
        dbg!(bravery_badge_event.tags.clone());
        let bravery_badge_award =
            BadgeAward::new(&bravery_badge_event, awarded_pub_keys.clone(), &keys)
                .unwrap()
                .0;

        let honor_badge_event = get_badge_with_id_only("honor".to_owned(), &keys).0;
        let honor_badge_award = BadgeAward::new(&honor_badge_event, awarded_pub_keys, &keys)
            .unwrap()
            .0;
        let badge_definitions = vec![bravery_badge_event, honor_badge_event];

        let badge_awards = vec![bravery_badge_award, honor_badge_award];
        dbg!(badge_awards.clone());

        assert_eq!(badge_awards.len(), 2);
        assert_eq!(badge_definitions.len(), 2);

        let profile_badges = ProfileBadgesEvent::new(badge_definitions, badge_awards, &keys)
            .unwrap()
            .0;
        dbg!(profile_badges.clone());

        dbg!(example_event.clone());

        assert_eq!(profile_badges.kind, Kind::ProfileBadges);
        assert!(true);
    }
}
