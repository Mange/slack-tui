use failure::{Error, Fail};

pub fn print_error_and_exit(error: Error) -> ! {
    for (i, cause) in error.causes().enumerate() {
        if i == 0 {
            eprintln!("Error: {}", cause);
        } else {
            let indentation = 4 * i;
            eprintln!("{0:1$}Caused by: {2}", "", indentation, cause);
        }

        #[cfg(debug_assertions)]
        {
            if let Some(backtrace) = cause.backtrace() {
                println!("{:#?}", backtrace);
            }
        }
    }
    eprintln!("\n...Sorry :(");

    ::std::process::exit(1);
}

pub fn format_error_with_causes<E: Fail>(error: E) -> String {
    error
        .causes()
        .enumerate()
        .map(|(i, cause)| {
            #[cfg(debug_assertions)]
            let backtrace = if let Some(backtrace) = cause.backtrace() {
                format!("\n{:#?}\n", backtrace)
            } else {
                String::new()
            };

            #[cfg(not(debug_assertions))]
            let backtrace = String::new();

            if i == 0 {
                format!("Error: {}{}", cause, backtrace)
            } else {
                let indentation = 4 * i;
                format!(
                    "\n{0:1$}Caused by: {2}{3}",
                    "", indentation, cause, backtrace
                )
            }
        })
        .collect()
}

#[cfg(test)]
extern crate tui;
#[cfg(test)]
pub fn render_buffer(buf: &tui::buffer::Buffer) -> String {
    let mut s = format!("Buffer area: {:?}\r\n", buf.area());
    let width = buf.area().width;
    for (i, cell) in buf.content().iter().enumerate() {
        if i > 0 && i as u16 % width == 0 {
            s.push_str("\r\n");
        }
        s.push(cell.symbol.chars().next().unwrap());
    }
    s
}
