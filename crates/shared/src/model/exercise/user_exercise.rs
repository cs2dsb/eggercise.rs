use chrono::{DateTime, Utc};

use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "user_exercise",
    "../../../migrations/005-user_exercise/up.sql",
    /// User specific overrides to an Exercise's parameters
    pub struct UserExercise {
        pub id: Uuid,
        pub exercise_id: Uuid,
        pub user_id: Uuid,
        /// How many days recovery are required between sets of this exercise
        pub recovery_days: Option<f64>,
        pub creation_date: DateTime<Utc>,
        pub last_updated_date: DateTime<Utc>,
    }
);

#[cfg(feature = "wasm")]
impl crate::model::model_into_view::UseDefaultModelView for UserExercise {}
