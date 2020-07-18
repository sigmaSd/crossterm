/// Append a the first few characters of an ANSI escape code to the given string.
#[macro_export]
#[doc(hidden)]
macro_rules! csi {
    ($( $l:expr ),*) => { concat!("\x1B[", $( $l ),*) };
}

/// Writes an ansi code to the given writer.
#[doc(hidden)]
#[macro_export]
macro_rules! write_ansi_code {
    ($writer:expr, $ansi_code:expr) => {{
        use std::io::{self, ErrorKind};

        write!($writer, "{}", $ansi_code)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))
            .map_err($crate::ErrorKind::IoError)
    }};
}

/// Writes/executes the given command.
#[doc(hidden)]
#[macro_export]
macro_rules! handle_command {
    ($writer:expr, $command:expr) => {{
        // Silent warning when the macro is used inside the `command` module
        #[allow(unused_imports)]
        use $crate::{write_ansi_code, Command};

        #[cfg(windows)]
        {
            let command = $command;
            if command.is_ansi_code_supported() {
                write_ansi_code!($writer, command.ansi_code())
            } else {
                command
                    .execute_winapi(|| {
                        write!($writer, "{}", command.ansi_code())?;
                        // winapi doesn't support queuing
                        $writer.flush()?;
                        Ok(())
                    })
                    .map_err($crate::ErrorKind::from)
            }
        }
        #[cfg(unix)]
        {
            write_ansi_code!($writer, $command.ansi_code())
        }
    }};
}

// Offer the same functionality as queue! macro, but is used only internally and with std::fmt::Write as $writer
// The difference is in case of winapi we ignore the $writer and use a fake one
#[doc(hidden)]
#[macro_export]
macro_rules! handle_fmt_command {
    ($writer:expr, $command:expr) => {{
        use $crate::{write_ansi_code, Command};

        #[cfg(windows)]
        {
            let command = $command;
            if command.is_ansi_code_supported() {
                write_ansi_code!($writer, command.ansi_code())
            } else {
                command
                    .execute_winapi(|| Ok(()))
                    .map_err($crate::ErrorKind::from)
            }
        }
        #[cfg(unix)]
        {
            write_ansi_code!($writer, $command.ansi_code())
        }
    }};
}

/// Queues one or more command(s) for further execution.
///
/// Queued commands must be flushed to the underlying device to be executed.
/// This generally happens in the following cases:
///
/// * When `flush` is called manually on the given type implementing `io::Write`.
/// * The terminal will `flush` automatically if the buffer is full.
/// * Each line is flushed in case of `stdout`, because it is line buffered.
///
/// # Arguments
///
/// - [std::io::Writer](https://doc.rust-lang.org/std/io/trait.Write.html)
///
///     ANSI escape codes are written on the given 'writer', after which they are flushed.
///
/// - [Command](./trait.Command.html)
///
///     One or more commands
///
/// # Examples
///
/// ```rust
/// use std::io::{Write, stdout};
/// use crossterm::{queue, style::Print};
///
/// fn main() {
///     let mut stdout = stdout();
///
///     // `Print` will executed executed when `flush` is called.
///     queue!(stdout, Print("foo".to_string()));
///
///     // some other code (no execution happening here) ...
///
///     // when calling `flush` on `stdout`, all commands will be written to the stdout and therefore executed.
///     stdout.flush();
///
///     // ==== Output ====
///     // foo
/// }
/// ```
///
/// Have a look over at the [Command API](./#command-api) for more details.
///
/// # Notes
///
/// In case of Windows versions lower than 10, a direct WinApi call will be made.
/// The reason for this is that Windows versions lower than 10 do not support ANSI codes,
/// and can therefore not be written to the given `writer`.
/// Therefore, there is no difference between [execute](macro.execute.html)
/// and [queue](macro.queue.html) for those old Windows versions.
///
#[macro_export]
macro_rules! queue {
    ($writer:expr $(, $command:expr)* $(,)?) => {
        Ok(()) $(
            .and_then(|()| $crate::handle_command!($writer, $command))
        )*
    }
}

/// Executes one or more command(s).
///
/// # Arguments
///
/// - [std::io::Writer](https://doc.rust-lang.org/std/io/trait.Write.html)
///
///     ANSI escape codes are written on the given 'writer', after which they are flushed.
///
/// - [Command](./trait.Command.html)
///
///     One or more commands
///
/// # Examples
///
/// ```rust
/// use std::io::{Write, stdout};
/// use crossterm::{execute, style::Print};
///
///  fn main() {
///      // will be executed directly
///      execute!(stdout(), Print("sum:\n".to_string()));
///
///      // will be executed directly
///      execute!(stdout(), Print("1 + 1= ".to_string()), Print((1+1).to_string()));
///
///      // ==== Output ====
///      // sum:
///      // 1 + 1 = 2
///  }
/// ```
///
/// Have a look over at the [Command API](./#command-api) for more details.
///
/// # Notes
///
/// * In the case of UNIX and Windows 10, ANSI codes are written to the given 'writer'.
/// * In case of Windows versions lower than 10, a direct WinApi call will be made.
///     The reason for this is that Windows versions lower than 10 do not support ANSI codes,
///     and can therefore not be written to the given `writer`.
///     Therefore, there is no difference between [execute](macro.execute.html)
///     and [queue](macro.queue.html) for those old Windows versions.
#[macro_export]
macro_rules! execute {
    ($writer:expr $(, $command:expr)* $(,)? ) => {
        // Queue each command, then flush
        $crate::queue!($writer $(, $command)*).and_then(|()| {
            $writer.flush().map_err($crate::ErrorKind::IoError)
        })
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_display {
    (for $($t:ty),+) => {
        $(impl ::std::fmt::Display for $t {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::result::Result<(), ::std::fmt::Error> {
                $crate::handle_fmt_command!(f, self).map_err(|_| ::std::fmt::Error)
            }
        })*
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_from {
    ($from:path, $to:expr) => {
        impl From<$from> for ErrorKind {
            fn from(e: $from) -> Self {
                $to(e)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::str;
    // Helper for execute tests to confirm flush
    #[derive(Default, Debug, Clone)]
    pub(self) struct FakeWrite {
        buffer: String,
        flushed: bool,
    }

    impl io::Write for FakeWrite {
        fn write(&mut self, content: &[u8]) -> io::Result<usize> {
            let content = str::from_utf8(content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            self.buffer.push_str(content);
            self.flushed = false;
            Ok(content.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushed = true;
            Ok(())
        }
    }

    #[cfg(not(windows))]
    mod unix {
        use std::io::Write;

        use super::FakeWrite;
        use crate::command::Command;

        pub struct FakeCommand;

        impl Command for FakeCommand {
            type AnsiType = &'static str;

            fn ansi_code(&self) -> Self::AnsiType {
                "cmd"
            }
        }

        #[test]
        fn test_queue_one() {
            let mut result = FakeWrite::default();
            queue!(&mut result, FakeCommand).unwrap();
            assert_eq!(&result.buffer, "cmd");
            assert!(!result.flushed);
        }

        #[test]
        fn test_queue_many() {
            let mut result = FakeWrite::default();
            queue!(&mut result, FakeCommand, FakeCommand).unwrap();
            assert_eq!(&result.buffer, "cmdcmd");
            assert!(!result.flushed);
        }

        #[test]
        fn test_queue_trailing_comma() {
            let mut result = FakeWrite::default();
            queue!(&mut result, FakeCommand, FakeCommand,).unwrap();
            assert_eq!(&result.buffer, "cmdcmd");
            assert!(!result.flushed);
        }

        #[test]
        fn test_execute_one() {
            let mut result = FakeWrite::default();
            execute!(&mut result, FakeCommand).unwrap();
            assert_eq!(&result.buffer, "cmd");
            assert!(result.flushed);
        }

        #[test]
        fn test_execute_many() {
            let mut result = FakeWrite::default();
            execute!(&mut result, FakeCommand, FakeCommand).unwrap();
            assert_eq!(&result.buffer, "cmdcmd");
            assert!(result.flushed);
        }

        #[test]
        fn test_execute_trailing_comma() {
            let mut result = FakeWrite::default();
            execute!(&mut result, FakeCommand, FakeCommand,).unwrap();
            assert_eq!(&result.buffer, "cmdcmd");
            assert!(result.flushed);
        }
    }

    #[cfg(windows)]
    mod windows {
        use std::cell::RefCell;
        use std::fmt::Debug;
        use std::io::Write;

        use super::FakeWrite;
        use crate::command::Command;
        use crate::error::Result as CrosstermResult;

        // We need to test two different APIs: winapi and the write api. We
        // don't know until runtime which we're supporting (via
        // Command::is_ansi_code_supported), so we have to test them both. The
        // CI environment hopefully includes both versions of windows.

        // WindowsEventStream is a place for execute_winapi to push strings,
        // when called.
        type WindowsEventStream = Vec<&'static str>;

        struct FakeCommand<'a> {
            // Need to use a refcell because we want execute_winapi to be able
            // push to the vector, but execute_winapi take &self.
            stream: RefCell<&'a mut WindowsEventStream>,
            value: &'static str,
        }

        impl<'a> FakeCommand<'a> {
            fn new(stream: &'a mut WindowsEventStream, value: &'static str) -> Self {
                Self {
                    value,
                    stream: RefCell::new(stream),
                }
            }
        }

        impl<'a> Command for FakeCommand<'a> {
            type AnsiType = &'static str;

            fn ansi_code(&self) -> Self::AnsiType {
                self.value
            }

            fn execute_winapi(
                &self,
                _writer: impl FnMut() -> CrosstermResult<()>,
            ) -> CrosstermResult<()> {
                self.stream.borrow_mut().push(self.value);
                Ok(())
            }
        }

        // Helper function for running tests against either winapi or an
        // io::Write.
        //
        // This function will execute the `test` function, which should
        // queue some commands against the given FakeWrite and
        // WindowsEventStream. It will then test that the correct data sink
        // was populated. It does not currently check is_ansi_code_supported;
        // for now it simply checks that one of the two streams was correctly
        // populated.
        //
        // If the stream was populated, it tests that the two arrays are equal.
        // If the writer was populated, it tests that the contents of the
        // write buffer are equal to the concatenation of `stream_result`.
        fn test_harness<E: Debug>(
            stream_result: &[&'static str],
            test: impl FnOnce(&mut FakeWrite, &mut WindowsEventStream) -> Result<(), E>,
        ) {
            let mut stream = WindowsEventStream::default();
            let mut writer = FakeWrite::default();

            if let Err(err) = test(&mut writer, &mut stream) {
                panic!("Error returned from test function: {:?}", err);
            }

            // We need this for type inference, for whatever reason.
            const EMPTY_RESULT: [&'static str; 0] = [];

            // TODO: confirm that the correct sink was used, based on
            // is_ansi_code_supported
            match (writer.buffer.is_empty(), stream.is_empty()) {
                (true, true) if stream_result == &EMPTY_RESULT => {}
                (true, true) => panic!(
                    "Neither the event stream nor the writer were populated. Expected {:?}",
                    stream_result
                ),

                // writer is populated
                (false, true) => {
                    // Concat the stream result to find the string result
                    let result: String = stream_result.iter().copied().collect();
                    assert_eq!(result, writer.buffer);
                    assert_eq!(&stream, &EMPTY_RESULT);
                }

                // stream is populated
                (true, false) => {
                    assert_eq!(stream, stream_result);
                    assert_eq!(writer.buffer, "");
                }

                // Both are populated
                (false, false) => panic!(
                    "Both the writer and the event stream were written to.\n\
                     Only one should be used, based on is_ansi_code_supported.\n\
                     stream: {stream:?}\n\
                     writer: {writer:?}",
                    stream = stream,
                    writer = writer,
                ),
            }
        }

        #[test]
        fn test_queue_one() {
            test_harness(&["cmd1"], |writer, stream| {
                queue!(writer, FakeCommand::new(stream, "cmd1"))
            })
        }

        #[test]
        fn test_queue_some() {
            test_harness(&["cmd1", "cmd2"], |writer, stream| {
                queue!(
                    writer,
                    FakeCommand::new(stream, "cmd1"),
                    FakeCommand::new(stream, "cmd2"),
                )
            })
        }

        #[test]
        fn test_many_queues() {
            test_harness(&["cmd1", "cmd2", "cmd3"], |writer, stream| {
                queue!(writer, FakeCommand::new(stream, "cmd1"))?;
                queue!(writer, FakeCommand::new(stream, "cmd2"))?;
                queue!(writer, FakeCommand::new(stream, "cmd3"))
            })
        }
    }
}
