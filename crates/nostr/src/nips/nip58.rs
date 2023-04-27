//! NIP58
//!
//! <https://github.com/nostr-protocol/nips/blob/master/58.md>

use crate::{event::builder::Error as BuilderError, Event, EventBuilder, Keys, Kind, Tag};

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
                Tag::ImageWithDimensions(image, width, height)
            } else {
                Tag::Image(image)
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
                    Tag::ThumbWithDimensions(thumb_url, width, height)
                } else {
                    Tag::Thumb(thumb_url)
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
    pub fn new(badge: Tag, awarded_pub_keys: Vec<Tag>, keys: &Keys) -> Result<BadgeAward, Error> {
        match badge {
            Tag::A { kind, .. } => {
                if kind != Kind::BadgeDefinition {
                    return Err(Error::InvalidKind);
                }
            }
            _ => return Err(Error::InvalidKind),
        };

        let awarded_pub_keys: Vec<Tag> = awarded_pub_keys
            .into_iter()
            .filter(|e| matches!(e, Tag::PubKey(..)))
            .collect();

        if awarded_pub_keys.is_empty() {
            return Err(Error::InvalidKind);
        }

        let mut tags = vec![badge];
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
    /// Helper function to filter tags for a specific [`Kind`]
    fn filter_for_kind(tags: Vec<Tag>, kind_needed: &Kind) -> Vec<Tag> {
        tags.into_iter()
            .filter(|e| match e {
                Tag::A { kind, .. } => kind == kind_needed,
                _ => false,
            })
            .collect()
    }

    /// Create a new [`ProfileBadgesEvent`] from badge definition and awards tags.
    /// [`badge_definitions`] and [`badge_awards`] must be ordered, so on the same position they refer to the same badge
    pub fn new(
        badge_definitions: Vec<Tag>,
        badge_awards: Vec<Tag>,
        keys: &Keys,
    ) -> Result<ProfileBadgesEvent, ProfileBadgesEventError> {
        if badge_definitions.len() != badge_awards.len() {
            return Err(ProfileBadgesEventError::InvalidLength);
        }

        let badge_awards = ProfileBadgesEvent::filter_for_kind(badge_awards, &Kind::BadgeAward);
        if badge_awards.is_empty() {
            return Err(ProfileBadgesEventError::InvalidKind);
        }

        let badge_definitions =
            ProfileBadgesEvent::filter_for_kind(badge_definitions, &Kind::BadgeDefinition);
        if badge_definitions.is_empty() {
            return Err(ProfileBadgesEventError::InvalidKind);
        }

        // Add identifier `d` tag
        let id_tag = Tag::Identifier("profile_badges".to_owned());
        let mut tags: Vec<Tag> = vec![id_tag];

        // This collection has been filtered for the needed tags
        let users_badges = core::iter::zip(badge_definitions, badge_awards);
        for (badge_definition, badge_award) in users_badges {
            match (&badge_definition, &badge_award) {
                (Tag::A { ref identifier, .. }, Tag::Identifier(ref badge_id))
                    if badge_id != identifier =>
                {
                    return Err(ProfileBadgesEventError::MismatchedBadgeDefinitionOrAward);
                }
                (Tag::A { identifier, .. }, Tag::Identifier(badge_id))
                    if badge_id == identifier =>
                {
                    tags.push(badge_award);
                    tags.push(badge_definition);
                }
                _ => {}
            }
        }

        // Badge definitions and awards have been validated

        let event_builder = EventBuilder::new(Kind::BadgeAward, String::new(), &tags);
        let event = event_builder.to_event(keys)?;

        Ok(ProfileBadgesEvent(event))
    }
}
