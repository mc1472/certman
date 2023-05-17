use std::process::exit;

use rustyline::DefaultEditor;

pub fn prompt_question(
    rl: &mut DefaultEditor,
    question: &str,
    true_answer: &str,
) -> anyhow::Result<bool> {
    let responce = rl.readline(question)?;
    Ok(responce.to_lowercase().contains(true_answer))
}

pub fn exit_with_msg(message: &str) -> ! {
    exit_with_msg_and_code(message, 1)
}

pub fn exit_with_msg_and_code(message: &str, code: i32) -> ! {
    eprintln!("{message}");
    exit(code)
}
