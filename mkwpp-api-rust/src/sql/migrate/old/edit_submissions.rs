use crate::api::errors::FinalErrorResponse;

#[derive(serde::Deserialize, Debug)]
pub struct EditSubmissions {
    video_link: Option<String>,
    ghost_link: Option<String>,
    comment: Option<String>,
    video_link_edited: bool,
    ghost_link_edited: bool,
    comment_edited: bool,
    admin_note: Option<String>,
    status: u8,
    submitted_by: i32,
    submitted_at: String,
    submitter_note: Option<String>,
    reviewed_by: Option<i32>,
    reviewed_at: Option<String>,
    reviewer_note: Option<String>,
    score: i32,
}

impl super::OldFixtureJson for EditSubmissions {
    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        return crate::sql::tables::submissions::edit_submissions::EditSubmissions {
            id: key,
            date: None,
            date_edited: false,
            video_link_edited: self.video_link_edited,
            ghost_link_edited: self.ghost_link_edited,
            comment_edited: self.comment_edited,
            video_link: self.video_link,
            ghost_link: self.ghost_link,
            comment: self.comment,
            admin_note: self.admin_note,
            status: crate::sql::tables::submissions::SubmissionStatus::try_from(self.status)
                .unwrap(),
            submitter_id: self.submitted_by,
            submitter_note: self.submitter_note,
            submitted_at: chrono::DateTime::from_naive_utc_and_offset(
                chrono::NaiveDateTime::parse_from_str(&self.submitted_at, "%FT%T%.3fZ").unwrap(),
                chrono::Utc,
            ),
            reviewer_id: self.reviewed_by,
            reviewer_note: self.reviewer_note,
            reviewed_at: self.reviewed_at.map(|v| {
                chrono::DateTime::from_naive_utc_and_offset(
                    chrono::NaiveDateTime::parse_from_str(&v, "%FT%T%.3fZ").unwrap(),
                    chrono::Utc,
                )
            }),
            score_id: self.score,
        }
        .insert_or_replace_query(transaction)
        .await;
    }
}
