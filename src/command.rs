use regex::Regex;

#[derive(Clone)]
pub struct Command {
    pub(crate) command: String,
    pub(crate) options: Vec<String>,
    pub(crate) line: String
}

impl Command {
    pub(crate) fn new(command: String, options: Vec<String>, line: String) -> Command {
        Command {
            command, options, line
        }
    }

    pub(crate) fn from_line(line: &str) -> Command {
        let command_regex = Regex::new(r"^([A-Z]+)").unwrap();
        let options_pre_regex = Regex::new(r"(^[A-Z]+ (?:[^-]{0,1}-[a-z0-9]|--[a-z-]+[ =]{0,1}[^ ]+| )+)").unwrap();
        let options_regex = Regex::new(r"([^-]{0,1}-[a-z0-9]| --[a-z-]+[ =]{0,1}[^ ]+)").unwrap();

        let command_captures = command_regex.captures(line).unwrap();
        let command_name = command_captures.get(1).unwrap().as_str().to_uppercase().to_string();
        let mut command_cut = String::from(line).replace(&command_name, "");

        let mut options: Vec<String> = Vec::new();

        match options_pre_regex.find(line) {
            Some(options_pre) => {
                for option_raw in options_regex.find_iter(options_pre.as_str()) {
                    let option = option_raw.as_str().trim();
                    options.push(String::from(option));
                    command_cut = command_cut.replace(option, "");
                }
            }
            None => {
            }
        }

        Command::new(
            String::from(command_name),
            options,
            String::from(command_cut.trim())
        )
    }

    pub fn format(&self) -> String {
        if self.options.is_empty() {
            return format!("{} {}", self.command, self.line);
        }
        format!("{} {} {}", self.command, self.options.join(" "), self.line)
    }
}

pub struct CommandCollection {
    pub(crate) commands: Vec<Command>
}

impl CommandCollection {
    pub(crate) fn new() -> CommandCollection {
        CommandCollection {
            commands: vec![]
        }
    }

    pub(crate) fn from_vec(commands: Vec<Command>) -> CommandCollection {
        CommandCollection {
            commands
        }
    }

    pub(crate) fn from_file(dockerfile_content: &str) -> CommandCollection {
        let mut commands: CommandCollection = CommandCollection::new();

        for line in CommandCollection::cleanup_file(dockerfile_content) {
            commands = commands.add(Command::from_line(&line));
        }

        commands
    }

    pub fn add(mut self, command: Command) -> CommandCollection {
        self.commands.push(command);

        self
    }

    pub fn format(&self) -> String {
        let mut result = String::new();

        for command in &self.commands {
            result.push_str(&format!("{}\n", command.format()));
        }

        result
    }

    fn cleanup_file(dockerfile_content: &str) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();

        let mut add_next_line_to_last_line = false;

        for line in dockerfile_content.lines() {
            let mut current_line: String = "".to_string();
            if add_next_line_to_last_line {
                current_line.push_str(&lines.pop().unwrap().trim());
                current_line.push_str(" ");
            }

            current_line.push_str(&str::replace(&line.to_owned(), "\\", "").trim());

            if current_line.trim().len() > 0
                && !current_line.trim().starts_with("#")
                && !current_line.trim().starts_with("//")
            {
                lines.push(current_line);
            }

            if line.trim().ends_with("\\") {
                add_next_line_to_last_line = true;
            } else {
                add_next_line_to_last_line = false;
            }
        }

        lines
    }
}