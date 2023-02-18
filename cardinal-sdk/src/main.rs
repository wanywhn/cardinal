#![feature(iter_array_chunks)]
mod consts;
mod disk_entry;
mod fs_visitor;
mod models;
mod schema;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use consts::*;
use crossbeam_channel::bounded;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel_migrations::MigrationHarness;
use std::time::Instant;

const DATABASE_URL: &str = std::env!("DATABASE_URL");

fn main() -> Result<()> {
    let _ = std::fs::remove_file(DATABASE_URL);
    let mut conn = SqliteConnection::establish(DATABASE_URL).with_context(|| {
        anyhow!(
            "Establish sqlite connection with url: `{}` failed.",
            DATABASE_URL
        )
    })?;
    conn.batch_execute(CONNECTION_PRAGMAS)
        .context("Run connection pragmas failed.")?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!(e))
        .context("Run connection migrations failed.")?;

    let (raw_entry_sender, raw_entry_receiver) = bounded(MAX_RAW_ENTRY_COUNT);

    std::thread::spawn(move || {
        let threads = num_cpus::get_physical();
        dbg!(threads);
        let walkdir = ignore::WalkBuilder::new("/")
            .follow_links(false)
            .git_exclude(false)
            .git_global(false)
            .git_ignore(false)
            .hidden(false)
            .ignore(false)
            .ignore_case_insensitive(false)
            .max_depth(None)
            .max_filesize(None)
            .parents(false)
            .require_git(false)
            .same_file_system(true)
            .skip_stdout(false)
            .standard_filters(false)
            .threads(threads)
            .build_parallel();
        let mut visitor_builder = fs_visitor::VisitorBuilder { raw_entry_sender };
        walkdir.visit(&mut visitor_builder);
    });

    let mut last_time = Instant::now();
    let mut insert_num = 0;
    let mut printed = 0;
    for entrys in raw_entry_receiver.iter() {
        if insert_num - printed >= 100000 {
            println!(
                "insert: {}, speed: {}i/s, remaining: {}",
                insert_num,
                (insert_num - printed) as f32 / last_time.elapsed().as_secs_f32(),
                raw_entry_receiver.len(),
            );
            last_time = Instant::now();
            printed = insert_num;
        }
        insert_num += entrys.len();
        conn.transaction(|conn| {
            use schema::dir_entrys::dsl::*;
            for entry in entrys {
                let _num_insert = diesel::insert_into(dir_entrys)
                    .values(&entry)
                    .on_conflict(the_path)
                    .do_update()
                    .set(the_meta.eq(&entry.the_meta))
                    .execute(conn)?;
            }
            Ok::<(), diesel::result::Error>(())
        })?;
    }

    Ok(())
}

/*
async fn add_row(
    conn: &mut SqliteConnection,
    DiskEntryRaw { the_path, the_meta }: DiskEntryRaw,
) -> Result<()> {
    sqlx::query!(
        r#"
INSERT INTO rows (the_path, the_meta)
VALUES (?,?)
ON CONFLICT(the_path) DO UPDATE SET the_meta = excluded.the_meta
        "#,
        the_path,
        the_meta
    )
    .execute(conn)
    .await
    .context("Upsert disk entry failed.")?;
    Ok(())
}

async fn get_row(pool: &SqlitePool, path: &[u8]) -> Result<DiskEntryRaw> {
    let mut conn = pool.acquire().await?;
    let row = sqlx::query_as!(
        DiskEntryRaw,
        r#"
SELECT the_path, the_meta
FROM rows
WHERE the_path = ?
        "#,
        path
    )
    .fetch_one(&mut conn)
    .await
    .context("Fetch from db failed.")?;
    Ok(row)
}

 */
