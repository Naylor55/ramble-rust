use crossterm::{ExecutableCommand, terminal, cursor, event, execute};
use crossterm::event::{KeyCode, KeyEvent};
use std::io::{Write, stdout};
use std::time::Duration;

fn main() {
    // 启用原始模式
    terminal::enable_raw_mode().unwrap();

    // 清屏并显示欢迎信息
    stdout().execute(terminal::Clear(terminal::ClearType::All)).unwrap();
    println!("Welcome to the terminal emulator!");

    // 进入循环，等待用户输入
    loop {
        print!("> ");
        stdout().flush().unwrap();

        // 检测按键
        if event::poll(Duration::from_secs(1)).unwrap() {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Esc => break, // 按 `Esc` 键退出
                    KeyCode::Enter => {
                        println!("\nYou pressed Enter!");
                    },
                    KeyCode::Char(c) => {
                        println!("\nYou typed: '{}'", c);
                    },
                    _ => {}
                }
            }
        }
    }

    // 退出原始模式并恢复终端
    terminal::disable_raw_mode().unwrap();
    println!("Goodbye!");
}
