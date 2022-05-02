#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        println!("{}", ansi_term::Colour::Red.paint(format!($($arg)*)));
    });
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        println!("{}", ansi_term::Colour::Yellow.paint(format!($($arg)*)));
    });
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        println!("{}", ansi_term::Colour::Blue.paint(format!($($arg)*)));
    });
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => ({
        println!($($arg)*);
    });
}

#[macro_export]
macro_rules! ok {
    ($($arg:tt)*) => ({
        println!("{}", ansi_term::Colour::Green.paint(format!($($arg)*)));
    });
}