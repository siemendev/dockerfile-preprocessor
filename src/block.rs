use std::collections::HashMap;
use regex::Regex;
use crate::command::{Command, CommandCollection};
use crate::preprocessor::Preprocessor;
use crate::variables::VariableCollection;

#[derive(Clone)]
pub struct Block {
    pub(crate) name: String,
    pub(crate) commands: Vec<Command>
}

impl Block {
    pub(crate) fn new(name: String, commands: Vec<Command>) -> Block {
        Block {
            name, commands
        }
    }
}

#[derive(Clone)]
pub struct BlockCollection {
    blocks: HashMap<String, Block>
}

impl BlockCollection {
    pub(crate) fn new() -> BlockCollection {
        BlockCollection {
            blocks: HashMap::new()
        }
    }

    pub(crate) fn from_file_content(file_content: &str, parent_blocks: &BlockCollection, resolver: &Preprocessor, current_context: &str, variables: &VariableCollection) -> BlockCollection {
        let block_regex = Regex::new(r"BLOCK ([A-Za-z0-9-_]+)\n((?s:.+?))ENDBLOCK").unwrap();

        let mut block_collection = BlockCollection::new();

        for part in block_regex.find_iter(file_content) {
            let found = block_regex.captures(part.as_str());
            let block_name: &str = found.as_ref().unwrap().get(1).unwrap().as_str();
            let block_content: &str = found.as_ref().unwrap().get(2).unwrap().as_str();

            let mut commands: Vec<Command> = vec![];

            if parent_blocks.block_exists(block_name) {
                for parent_command in parent_blocks.get_block_commands(block_name) {
                    if parent_command.command == "PARENT" {
                        for command in CommandCollection::from_file(block_content).commands {
                            for handled_command in resolver.handle_command(&command, current_context, variables).commands {
                                commands.push(handled_command);
                            }
                        }
                        continue;
                    }
                    commands.push(parent_command);
                }

                block_collection = block_collection.add_block(
                    Block::new(String::from(block_name), commands)
                );

                continue;
            }

            for command in CommandCollection::from_file(block_content).commands {
                for handled_command in resolver.handle_command(&command, current_context, variables).commands {
                    commands.push(handled_command);
                }
            }

            block_collection = block_collection.add_block(
                Block::new(String::from(block_name), commands)
            );
        }

        block_collection
    }

    pub(crate) fn add_block(mut self, block: Block) -> BlockCollection {
        self.blocks.insert(String::from(&block.name), block);

        self
    }

    pub(crate) fn block_exists(&self, block_name: &str) -> bool {
        self.blocks.contains_key(block_name)
    }

    pub(crate) fn get_block_commands(&self, block_name: &str) -> Vec<Command> {
        let mut commands = vec![];

        if !self.block_exists(block_name) {
            return commands;
        }

        for command in &self.blocks.get(block_name).unwrap().commands {
            commands.push(command.clone());
        }

        commands
    }
}