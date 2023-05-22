use rustyline::DefaultEditor;

pub fn prompt_question(
    rl: &mut DefaultEditor,
    question: &str,
    true_answer: &str,
) -> anyhow::Result<bool> {
    let responce = rl.readline(question)?;
    Ok(responce.to_lowercase().contains(true_answer))
}
