use crate::api::errors::FinalErrorResponse;

mod awards;
mod blog_posts;
mod champs;
mod edit_submissions;
mod players;
mod regions;
mod scores;
mod standard_levels;
mod standards;
mod submissions;
mod tracks;

const OLD_FIXTURES_PATH: &str = "./db/fixtures/old/";

fn enforce_file_order(file_name: &str) -> u8 {
    match file_name {
        "regions.json" => 0,
        "players.json" => 1,
        "trackcups.json" => 2,
        "tracks.json" => 3,
        "scores.json" => 4,
        "scoresubmissions.json" => 5,
        "editscoresubmissions.json" => 6,
        "standardlevels.json" => 7,
        "standards.json" => 8,
        "sitechampions.json" => 9,
        "playerawards.json" => 10,
        "blogposts.json" => 11,
        _ => 12,
    }
}

pub async fn load_data(pool: &sqlx::Pool<sqlx::Postgres>) {
    let transaction = pool.begin();

    let mut file_paths = match std::fs::read_dir(std::path::Path::new(OLD_FIXTURES_PATH)) {
        Err(e) => {
            println!("Error reading folder for fixtures");
            println!("{e}");
            println!();
            println!("Exiting the process");
            std::process::exit(0);
        }
        Ok(dir_read) => dir_read
            .into_iter()
            .filter_map(|dir_entry_result| match dir_entry_result {
                Err(e) => {
                    println!("Error reading file from folder for fixtures");
                    println!("{e}");
                    println!();
                    println!("Exiting the process");
                    std::process::exit(0);
                }
                Ok(file) => match file.file_name().to_str() {
                    None => {
                        println!("Error reading file path from folder for fixtures");
                        println!();
                        println!("Exiting the process");
                        std::process::exit(0);
                    }
                    Some(path) => {
                        if !path.ends_with(".json") {
                            return None;
                        }
                        Some(String::from(path))
                    }
                },
            })
            .collect::<Vec<String>>(),
    };

    file_paths.sort_by_key(|a| enforce_file_order(a));

    let mut transaction = match transaction.await {
        Ok(v) => v,
        Err(e) => {
            println!("Couldn't start Postgres Transaction");
            println!("{e}");
            println!();
            println!("Exiting the process");
            std::process::exit(0);
        }
    };

    for file_name in file_paths {
        println!("Loading fixture {file_name}");
        if let Err(e) = match file_name.as_str() {
            "regions.json" => {
                regions::Regions::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "players.json" => {
                players::Players::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "tracks.json" => {
                tracks::Tracks::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "scores.json" => {
                scores::Scores::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "blogposts.json" => {
                blog_posts::BlogPosts::read_file(&file_name, &mut String::new(), &mut transaction)
                    .await
            }
            "scoresubmissions.json" => {
                println!("Fixture file skipped because it can't be imported");
                continue;
                // submissions::Submissions::read_file(
                //     &file_name,
                //     &mut String::new(),
                //     &mut transaction,
                // )
                // .await
            }
            "editscoresubmissions.json" => {
                println!("Fixture file skipped because it can't be imported");
                continue;
                // edit_submissions::EditSubmissions::read_file(
                //     &file_name,
                //     &mut String::new(),
                //     &mut transaction,
                // )
                // .await
            }
            "standardlevels.json" => {
                standard_levels::StandardLevels::read_file(
                    &file_name,
                    &mut String::new(),
                    &mut transaction,
                )
                .await
            }
            "standards.json" => {
                standards::Standards::read_file(&file_name, &mut String::new(), &mut transaction)
                    .await
            }
            "sitechampions.json" => {
                champs::Champs::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            "playerawards.json" => {
                awards::Awards::read_file(&file_name, &mut String::new(), &mut transaction).await
            }
            _ => {
                println!("Fixture file skipped");
                continue;
            }
        } {
            println!("Error reading data. Rolling back transaction.");
            println!("{e}");
            match transaction.rollback().await {
                Ok(_) => std::process::exit(0),
                Err(e) => println!("Error rolling back transaction. You're fucked. :)\n{e}"),
            };
            std::process::exit(0);
        }
    }

    sqlx::query(
        "SELECT setval('regions_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM regions));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for regions");

    sqlx::query(
        "SELECT setval('blog_posts_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM blog_posts));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for blog_posts");

    sqlx::query("SELECT setval('edit_submission_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM edit_submissions));").execute(&mut *transaction).await.expect("Should've reset the minimum id for edit_submissions");

    sqlx::query("SELECT setval('player_awards_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM player_awards));").execute(&mut *transaction).await.expect("Should've reset the minimum id for player_awards");

    sqlx::query(
        "SELECT setval('players_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM players));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for players");

    sqlx::query(
        "SELECT setval('players_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM players));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for players");

    sqlx::query("SELECT setval('scores_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM scores));")
        .execute(&mut *transaction)
        .await
        .expect("Should've reset the minimum id for scores");

    sqlx::query(
        "SELECT setval('site_champs_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM site_champs));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for site_champs");

    sqlx::query("SELECT setval('standard_levels_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM standard_levels));").execute(&mut *transaction).await.expect("Should've reset the minimum id for standard_levels");

    sqlx::query(
        "SELECT setval('standards_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM standards));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for standards");

    sqlx::query(
        "SELECT setval('submissions_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM submissions));",
    )
    .execute(&mut *transaction)
    .await
    .expect("Should've reset the minimum id for submissions");

    sqlx::query("SELECT setval('tracks_id_seq', (SELECT COALESCE(MAX(id),1) AS id FROM tracks));")
        .execute(&mut *transaction)
        .await
        .expect("Should've reset the minimum id for tracks");

    match transaction.commit().await {
        Ok(_) => println!("Transaction went through!"),
        Err(e) => {
            println!("Transaction failed.\n{e}\nExiting the process");
            std::process::exit(0)
        }
    }
}

trait OldFixtureJson: std::fmt::Debug {
    // buffer is because of lifetime. You can't declare the string within the function sadly enough.
    async fn read_file<'d>(
        file_name: &str,
        buffer: &'d mut String,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<(), FinalErrorResponse>
    where
        Self: Sized + serde::Deserialize<'d> + std::marker::Sync + std::marker::Send,
    {
        let file_path = format!("{OLD_FIXTURES_PATH}{file_name}");
        match std::fs::read_to_string(&file_path) {
            Ok(v) => *buffer = v,
            Err(e) => {
                println!("Error reading file {file_path} from old fixtures");
                println!("{e}");
                println!();
                println!("Exiting the process");
                std::process::exit(0);
            }
        };
        let mut vec: Vec<OldFixtureWrapper<Self>> = match serde_json::from_str(buffer) {
            Ok(v) => v,
            Err(e) => {
                println!("Error converting fixture {file_path} from JSON");
                println!("{e}");
                println!();
                println!("Exiting the process");
                std::process::exit(0);
            }
        };

        vec.sort_by(Self::get_sort());

        for wrapper in vec {
            wrapper.add_to_db(transaction).await?;
        }

        Ok(())
    }

    fn get_sort()
    -> impl FnMut(&OldFixtureWrapper<Self>, &OldFixtureWrapper<Self>) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        |a, b| a.pk.cmp(&b.pk)
    }

    async fn add_to_db(
        self,
        key: i32,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse>;
}

#[derive(serde::Deserialize, Debug)]
struct OldFixtureWrapper<T: OldFixtureJson> {
    pk: i32,
    fields: T,
}

impl<T: OldFixtureJson + std::marker::Sync + std::marker::Send> OldFixtureWrapper<T> {
    async fn add_to_db(
        self,
        transaction: &mut sqlx::PgConnection,
    ) -> Result<sqlx::postgres::PgQueryResult, FinalErrorResponse> {
        self.fields.add_to_db(self.pk, transaction).await
    }
}
