use super::*;

#[test]
fn a_claude_transcript_reported_under_another_project_reopens_from_where_it_is() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let projects = dir.path().join("projects");
    let started = projects.join("-start").join("c.jsonl");
    std::fs::create_dir_all(started.parent().unwrap())?;
    std::fs::create_dir_all(projects.join("-moved"))?;
    std::fs::write(&started, format!("{PROMPT}\n"))?;
    let store = Store::open(dir.path())?;
    conversation(
        &store,
        "b",
        "claude",
        &projects.join("-moved").join("c.jsonl"),
    )?;
    store.conn().execute_batch(
        "INSERT INTO block (lineage, instance_id, fg_pid, status, status_pid, status_name)
           VALUES ('b', 'i1', 10, 'idle', 10, 'claude')",
    )?;
    let planned = candidate(store.conn(), store.dir(), "i2", "b")?;
    assert_eq!(planned.start, Start::Agent, "{}", planned.plan.reason);
    assert_eq!(
        planned.plan.transcript.as_deref(),
        Some(&*started.to_string_lossy())
    );
    Ok(())
}

#[test]
fn a_transcript_not_found_in_the_projects_folder_keeps_the_shell() -> Result<()> {
    // `elsewhere` holds a file of the same name, but outside Claude Code's projects folder
    let start = |root: &str, written: bool| -> Result<Start> {
        let dir = tempfile::tempdir()?;
        let folder = dir.path().join(root);
        std::fs::create_dir_all(folder.join("-start"))?;
        if written {
            std::fs::write(folder.join("-start").join("c.jsonl"), format!("{PROMPT}\n"))?;
        }
        let store = Store::open(dir.path())?;
        conversation(
            &store,
            "b",
            "claude",
            &folder.join("-moved").join("c.jsonl"),
        )?;
        store.conn().execute_batch(
            "INSERT INTO block (lineage, instance_id, fg_pid, status, status_pid, status_name)
               VALUES ('b', 'i1', 10, 'idle', 10, 'claude')",
        )?;
        Ok(candidate(store.conn(), store.dir(), "i2", "b")?.start)
    };
    assert_eq!(start("projects", false)?, Start::Shell);
    assert_eq!(start("elsewhere", true)?, Start::Shell);
    Ok(())
}
