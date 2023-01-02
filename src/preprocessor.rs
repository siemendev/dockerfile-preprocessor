use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::{Captures, Regex};
use crate::command::{Command, CommandCollection};
use crate::block::BlockCollection;
use crate::variables::VariableCollection;

pub struct Preprocessor {
    pub root_dir: String,
}

impl Preprocessor {
    pub fn new(root_dir: String) -> Preprocessor {
        Preprocessor {
            root_dir
        }
    }

    pub fn process_file(file: &str, variables: VariableCollection, root_dir: &str) -> CommandCollection {
        Preprocessor::new(String::from(root_dir)).process(
            Path::new(file).file_name().unwrap().to_str().unwrap(),
            &BlockCollection::new(),
            &Preprocessor::get_absolute_path_context(file),
            &variables,
        )
    }

    fn process(&self, target: &str, blocks: &BlockCollection, parent_context: &str, variables: &VariableCollection) -> CommandCollection {
        let current_context = Preprocessor::get_absolute_path_context(
            Path::new(parent_context).join(target).to_str().unwrap()
        );
        let file_content: String = self.get_file_content(parent_context, target);
        let commands_collection: CommandCollection = CommandCollection::from_file(&file_content);
        let mut commands_result: CommandCollection = CommandCollection::new();

        let mut current_block_name: String = "".to_string();
        let mut current_block_commands: Vec<Command> = Vec::new();

        for command in &commands_collection.commands {
            if command.command == "EXTEND" {
                let base_commands: CommandCollection = self.process(
                    &command.line,
                    &BlockCollection::from_file_content(&file_content, blocks, self, &current_context, variables),
                    &current_context,
                    variables,
                );

                for base_command in base_commands.commands {
                    commands_result = commands_result.add(base_command);
                }

                break;
            }
            if command.command == "BLOCK" {
                current_block_name = command.line.clone();
                current_block_commands.clear();
                continue;
            }
            if command.command == "ENDBLOCK" {
                if blocks.block_exists(&current_block_name) {
                    for parent_block_command in blocks.get_block_commands(&current_block_name) {
                        if parent_block_command.command == "PARENT" {
                            for current_block_command in current_block_commands.clone() {
                                commands_result = commands_result.add(current_block_command.clone());
                            }
                            continue;
                        }
                        commands_result = commands_result.add(parent_block_command.clone());
                    }
                } else {
                    for current_block_command in current_block_commands.clone() {
                        commands_result = commands_result.add(current_block_command.clone());
                    }
                }

                current_block_name = "".to_string();
                current_block_commands.clear();
                continue;
            }

            if !current_block_name.is_empty() {
                for resolved_command in self.handle_command(command, &current_context, variables).commands {
                    current_block_commands.push(resolved_command.clone());
                }
                continue;
            }

            for resolved_command in self.handle_command(command, &current_context, variables).commands {
                commands_result = commands_result.add(resolved_command.clone());
            }
        }

        return commands_result;
    }

    pub fn handle_command(&self, old_command: &Command, current_context: &str, variables: &VariableCollection) -> CommandCollection {
        let mut new_command = old_command.clone();

        if new_command.command == "INCLUDE" {
            let line_regex = Regex::new(r"(.*?)[\n\r\s]*WITH[\n\r\s]*\{(.*?)\}[\n\r\s]*(ONLY)*").unwrap();
            let line_captures = line_regex.captures(&new_command.line);
            let mut file = String::from(&new_command.line);
            let mut new_variables = variables.clone();

            if line_captures.is_some() {
                file = String::from(line_captures.as_ref().unwrap().get(1).unwrap().as_str());
                // when ONLY is set
                if line_captures.as_ref().unwrap().get(3).is_some() {
                    new_variables = VariableCollection::new();
                }
                // json content
                if line_captures.as_ref().unwrap().get(2).is_some() {
                    // todo maybe use a json parser for this instead of regular expressions (json needs to be parsed to a flat array though)
                    let variables_regex = Regex::new(r#""([^"?]+)"[\n\r\s]*:[\n\r\s]*"([^"?]+)""#).unwrap();
                    for capture in variables_regex.find_iter(line_captures.as_ref().unwrap().get(2).unwrap().as_str()) {
                        let variable_captures = variables_regex.captures(capture.as_str()).unwrap();
                        new_variables = new_variables.set(
                            variable_captures.get(1).unwrap().as_str(),
                            variable_captures.get(2).unwrap().as_str(),
                        );
                    }
                }
            }

            return self.process(
                &file,
                &BlockCollection::new(),
                current_context,
                &new_variables,
            );
        }
        if new_command.command == "COPY" || new_command.command == "ADD" {
            let split: Vec<&str> = new_command.line.split(" ").collect();
            let mut source = self.handle_copy(split[0], current_context);

            source.push_str(" ");
            source.push_str(&split[1..].join(" "));
            new_command.line = String::from(source.trim());
        }

        for i in 0..new_command.options.len() {
            new_command.options[i] = self.replace_variables(&new_command.options[i], variables);
        }
        new_command.line = self.replace_variables(&new_command.line, variables);

        return CommandCollection::from_vec(vec![new_command]);
    }

    fn replace_variables(&self, input: &str, variables: &VariableCollection) -> String {
        let regex = Regex::new(r"\{\{[\n\r\s]*(.*?)[\n\r\s]*\}\}").unwrap();

        regex.replace_all(input, |captures: &Captures| self.handle_capture(captures.get(1).unwrap().as_str(), variables)).to_string()
    }
    fn handle_capture(&self, capture: &str, variables: &VariableCollection) -> String
    {
        if !variables.has(capture) {
            // todo add backtrace
            panic!("Variable '{}' not found. Available variables: {}", capture, variables.keys().join(", "))
        }
        variables.get(capture).to_string()
    }

    pub fn get_absolute_path_context(file_path: &str) -> String {
        let path = Path::new(file_path);

        if path.exists() {
            return fs::canonicalize(file_path).unwrap().parent().unwrap().to_str().unwrap().to_owned();
        }

        panic!("Unable to find handler for target '{}'.", path.to_str().unwrap());
    }

    fn handle_copy(&self, file: &str, context: &str) -> String {
        let file_path = Path::new(context).join(file);
        let absolute_path = file_path.canonicalize().unwrap().to_str().unwrap().to_string();

        if !absolute_path.starts_with(&self.root_dir) {
            panic!("File '{}' is out of root context '{}'", absolute_path, &self.root_dir)
        }

        let mut root_clone = String::from(&self.root_dir);
        root_clone.push_str("/");

        absolute_path.replace(&root_clone, "")
    }

    fn get_file_content(&self, context: &str, target: &str) -> String {
        let path = Path::new(context).join(target);

        if path.exists() {
            match Preprocessor::read_file_to_string(&path) {
                Ok(contents) => return contents,
                Err(err) => panic!("Unable to read '{}': {}", target, err),
            }
        }

        panic!("Unable to find handler for target '{}'.", path.to_str().unwrap());
    }

    fn read_file_to_string<P: AsRef<Path>>(file: P) -> Result<String, std::io::Error> {
        let mut file = File::open(file)?;

        let mut data = String::new();
        file.read_to_string(&mut data)?;
        Ok(data)
    }
}
