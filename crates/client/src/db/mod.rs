use std::{any::type_name, future::Future};

use leptos::{create_local_resource, Resource};
use shared::model::{
    model_into_view::{ListOfModel, ModelIntoView},
    Model,
};

use crate::utils::sqlite3::{ExecResult, SqlitePromiser, SqlitePromiserError};

pub mod migrations;
pub mod model;

pub trait PromiserFetcher: Model + Clone + ModelIntoView + Send {
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
    fn fetch_by<T: Into<sea_query::Value>>(
        id: T,
        column: <Self as Model>::Iden,
    ) -> impl Future<Output = Result<Vec<Self>, SqlitePromiserError>> {
        let sql = Self::fetch_by_column_sql(id, column, false);

        let promiser = SqlitePromiser::use_promiser();
        async move {
            let result = promiser.exec(sql).await?;

            Self::extract_fields(result)
        }
    }
    fn fetch_one_by<T: Into<sea_query::Value>>(
        id: T,
        column: <Self as Model>::Iden,
    ) -> impl Future<Output = Result<Self, SqlitePromiserError>> {
        let sql = Self::fetch_by_column_sql(id, column, true);

        let promiser = SqlitePromiser::use_promiser();
        async move {
            let result = promiser.exec(sql).await?;
            let mut results = Self::extract_fields(result)?;

            if results.len() != 1 {
                Err(SqlitePromiserError::ExecResult(format!(
                    "Expected exactly 1 {} but got {}",
                    type_name::<Self>(),
                    results.len()
                )))
            } else {
                Ok(results.pop().unwrap())
            }
        }
    }
    fn fetch_all() -> impl Future<Output = Result<Vec<Self>, SqlitePromiserError>> {
        let sql = Self::fetch_all_sql();

        let promiser = SqlitePromiser::use_promiser();
        async move {
            let result = promiser.exec(sql).await?;

            Self::extract_fields(result)
        }
    }
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
