use crate::{feature_model_derives, feature_model_imports, types::Uuid};

feature_model_imports!();

feature_model_derives!(
    "exercise_group_member",
    "../../../migrations/007-exercise_group/up.sql",
    pub struct ExerciseGroupMember {
        pub id: Uuid,
        pub exercise_id: Uuid,
        pub group_id: Uuid,
    }
);

#[cfg(feature = "sea-query-enum")]
const EXERCISE_GROUP_MEMBER_STAR: [ExerciseGroupMemberIden; 3] = [
    ExerciseGroupMemberIden::Id,
    ExerciseGroupMemberIden::ExerciseId,
    ExerciseGroupMemberIden::GroupId,
];

#[cfg(feature = "backend")]
impl ExerciseGroupMember {
    pub fn fetch_by_id(
        conn: &Connection,
        id: &Uuid,
    ) -> Result<ExerciseGroupMember, rusqlite::Error> {
        let (sql, values) = Query::select()
            .columns(EXERCISE_GROUP_MEMBER_STAR)
            .from(ExerciseGroupMemberIden::Table)
            .and_where(Expr::col(ExerciseGroupMemberIden::Id).eq(id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut stmt = conn.prepare_cached(&sql)?;
        let res = stmt.query_row(&*values.as_params(), ExerciseGroupMember::from_row)?;
        Ok(res)
    }
}
