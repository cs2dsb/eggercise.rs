use leptos::{create_local_resource, Resource};
use shared::model::{
    model_into_view::{ListOfModel, ModelIntoView},
    Model,
};

use crate::utils::sqlite3::{ExecResult, SqlitePromiser, SqlitePromiserError};

pub mod migrations;
pub mod model;

pub trait PromiserFetcher: Model + Clone + ModelIntoView {
    fn all_resource() -> Resource<(), Result<ListOfModel<Self>, SqlitePromiserError>> {
        create_local_resource(
            || (),
            |_| async {
                let promiser = SqlitePromiser::use_promiser();

                let result = promiser.exec(Self::fetch_all_sql()).await?;

                let rows = Self::extract_fields(result)?;

                let model_rows = ListOfModel(rows);

                Ok(model_rows)
            },
        )
    }

    fn extract_fields(result: ExecResult) -> Result<Vec<Self>, SqlitePromiserError>;
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use model_into_view::ModelIntoView;
    use shared::model::*;

    use super::PromiserFetcher;

    #[test]
    fn test_exercise_is_promiser_fetcher() {
        fn t1<T: Model + Clone + ModelIntoView>(_t: PhantomData<T>) {}
        fn t2<T: PromiserFetcher>(_t: PhantomData<T>) {}
        t1::<Exercise>(PhantomData);
        t2::<Exercise>(PhantomData);
    }

    #[test]
    fn test_exercise_group_is_promiser_fetcher() {
        fn t1<T: Model + Clone + ModelIntoView>(_t: PhantomData<T>) {}
        fn t2<T: PromiserFetcher>(_t: PhantomData<T>) {}
        t1::<ExerciseGroup>(PhantomData);
        t2::<ExerciseGroup>(PhantomData);
    }

    #[test]
    fn test_exercise_group_member_is_promiser_fetcher() {
        fn t1<T: Model + Clone + ModelIntoView>(_t: PhantomData<T>) {}
        fn t2<T: PromiserFetcher>(_t: PhantomData<T>) {}
        t1::<ExerciseGroupMember>(PhantomData);
        t2::<ExerciseGroupMember>(PhantomData);
    }

    #[test]
    fn test_session_is_promiser_fetcher() {
        fn t1<T: Model + Clone + ModelIntoView>(_t: PhantomData<T>) {}
        fn t2<T: PromiserFetcher>(_t: PhantomData<T>) {}
        t1::<Session>(PhantomData);
        t2::<Session>(PhantomData);
    }

    #[test]
    fn test_session_exercise_is_promiser_fetcher() {
        fn t1<T: Model + Clone + ModelIntoView>(_t: PhantomData<T>) {}
        fn t2<T: PromiserFetcher>(_t: PhantomData<T>) {}
        t1::<SessionExercise>(PhantomData);
        t2::<SessionExercise>(PhantomData);
    }

    #[test]
    fn test_user_is_promiser_fetcher() {
        fn t1<T: Model + Clone + ModelIntoView>(_t: PhantomData<T>) {}
        fn t2<T: PromiserFetcher>(_t: PhantomData<T>) {}
        t1::<User>(PhantomData);
        t2::<User>(PhantomData);
    }
}
