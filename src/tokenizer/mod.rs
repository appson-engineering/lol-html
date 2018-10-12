#[macro_use]
mod state_machine_dsl;

#[macro_use]
mod syntax;

#[macro_use]
mod tag_name;

mod input_chunk;
mod lex_unit;
mod tree_builder_simulator;

use self::input_chunk::InputChunk;
pub use self::lex_unit::*;
pub use self::tag_name::TagName;
use self::tree_builder_simulator::*;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "testing_api")]
pub use self::tree_builder_simulator::{TextParsingMode, TextParsingModeSnapshot};

const DEFAULT_ATTR_BUFFER_CAPACITY: usize = 256;

pub trait LexUnitHandler {
    fn handle(&mut self, lex_unit: &LexUnit);
}

#[cfg(feature = "testing_api")]
impl<F: FnMut(&LexUnit)> LexUnitHandler for F {
    fn handle(&mut self, lex_unit: &LexUnit) {
        self(lex_unit);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TokenizerBailoutReason {
    BufferCapacityExceeded,
    TextParsingAmbiguity,
    MaxTagNestingReached,
}

pub type TokenizerState<H> =
    fn(&mut Tokenizer<H>, Option<u8>) -> Result<(), TokenizerBailoutReason>;

pub struct Tokenizer<H> {
    input_chunk: InputChunk,
    pos: usize,
    raw_start: usize,
    token_part_start: usize,
    finished: bool,
    state_enter: bool,
    allow_cdata: bool,
    lex_unit_handler: H,
    state: TokenizerState<H>,
    current_token: Option<TokenView>,
    current_attr: Option<AttributeView>,
    last_start_tag_name_hash: Option<u64>,
    closing_quote: u8,
    attr_buffer: Rc<RefCell<Vec<AttributeView>>>,
    tree_builder_simulator: TreeBuilderSimulator,

    #[cfg(feature = "testing_api")]
    text_parsing_mode_change_handler: Option<Box<TextParsingModeChangeHandler>>,
}

define_state_machine!();

impl<H: LexUnitHandler> Tokenizer<H> {
    pub fn new(buffer_capacity: usize, lex_unit_handler: H) -> Self {
        Tokenizer {
            input_chunk: InputChunk::new(buffer_capacity),
            pos: 0,
            raw_start: 0,
            token_part_start: 0,
            finished: false,
            state_enter: true,
            allow_cdata: false,
            lex_unit_handler,
            state: Tokenizer::data_state,
            current_token: None,
            current_attr: None,
            last_start_tag_name_hash: None,
            closing_quote: b'"',
            attr_buffer: Rc::new(RefCell::new(Vec::with_capacity(
                DEFAULT_ATTR_BUFFER_CAPACITY,
            ))),
            tree_builder_simulator: TreeBuilderSimulator::default(),

            #[cfg(feature = "testing_api")]
            text_parsing_mode_change_handler: None,
        }
    }

    pub fn tokenize_chunk(&mut self, chunk: &[u8]) -> Result<(), TokenizerBailoutReason> {
        self.input_chunk.write(chunk)?;

        while !self.finished {
            let ch = self.input_chunk.peek_at(self.pos);

            (self.state)(self, ch)?;

            self.pos += 1;
        }

        Ok(())
    }

    pub fn end(&self) {
        // TODO
    }

    #[cfg(feature = "testing_api")]
    pub fn set_state(&mut self, state: TokenizerState<H>) {
        self.state = state;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        self.last_start_tag_name_hash = name_hash;
    }

    #[cfg(feature = "testing_api")]
    pub fn set_text_parsing_mode_change_handler(
        &mut self,
        handler: Box<TextParsingModeChangeHandler>,
    ) {
        self.text_parsing_mode_change_handler = Some(handler);
    }
}
