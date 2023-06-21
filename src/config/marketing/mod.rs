pub use analytics::AnalyticsConfig;
use schemars::JsonSchema;
use serde::Deserialize;
pub use social::{SocialConfig, SocialLayer};

use super::ApplyLayer;

mod analytics;
mod social;

/// Marketing config (complete version)
#[derive(Debug)]
pub struct MarketingConfig {
    /// Analytics
    pub analytics: Option<AnalyticsConfig>,
    /// Social media
    pub social: SocialConfig,
}
/// Marketing config (partial version used by oranda.json)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MarketingLayer {
    /// Analytics
    pub analytics: Option<AnalyticsConfig>,
    /// Social media
    pub social: Option<SocialLayer>,
}

impl Default for MarketingConfig {
    fn default() -> Self {
        MarketingConfig {
            analytics: None,
            social: SocialConfig::default(),
        }
    }
}
impl ApplyLayer for MarketingConfig {
    type Layer = MarketingLayer;
    fn apply_layer(&mut self, layer: Self::Layer) {
        // This is intentionally written slightly cumbersome to make you update this
        let MarketingLayer { analytics, social } = layer;

        // FIXME: this is kinda goofy but there's not an obvious thing to do
        // if we need to change the enum variant and we care about preserving things.
        // So we just clobber the old value no matter what
        if let Some(analytics) = analytics {
            self.analytics = Some(analytics);
        }
        self.social.apply_val_layer(social);
    }
}
