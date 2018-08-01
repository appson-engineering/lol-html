#[macro_use]
mod helpers;

#[macro_use]
mod state_transition;

macro_rules! action {

    // Token emission
    //--------------------------------------------------------------------
    ( | $self:tt |> emit_eof ) => {
        action_helper!(@emit_lex_result |$self|> ShallowToken::Eof, None);
        $self.finished = true;
    };

    ( | $self:tt |> emit_chars ) => {
        if $self.pos > $self.raw_start {
            // NOTE: unlike any other tokens, character tokens don't have
            // any lexical symbols that determine their bounds. Therefore,
            // representation of character token content is the raw slice.
            // Also, we always emit characters if we encounter some other bounded
            // lexical structure and, thus, we use exclusive range for the raw slice.
            action_helper!(@emit_lex_result_with_raw_exclusive |$self|> ShallowToken::Character);
        }
    };

    ( | $self:tt |> emit_comment ) => {
        let mut text = SliceRange::default();

        action_helper!(@set_token_part_range |$self|> text);
        action_helper!(@emit_lex_result_with_raw_inclusive |$self|> ShallowToken::Comment(text));
    };

    ( | $self: ident |> emit_current_token ) => {
        match $self.current_token.take() {
            Some(token) => {
                action_helper!(@emit_lex_result_with_raw_inclusive |$self|> token);
            }
            None => unreachable!("Current token should exist at this point")
        }
    };


    // Slices
    //--------------------------------------------------------------------
    ( | $self:tt |> start_raw ) => {
        $self.raw_start = $self.pos;
    };

    ( | $self:tt |> start_token_part ) => {
        $self.token_part_start = $self.pos - $self.raw_start;
    };


    // Token creation
    //--------------------------------------------------------------------
    ( | $self:tt |> create_start_tag ) => {
        $self.attr_buffer.borrow_mut().clear();

        $self.current_token = Some(ShallowToken::StartTag {
            name: SliceRange::default(),
            attributes: Rc::clone(&$self.attr_buffer),
            self_closing: false,
        });
    };

    ( | $self:tt |> create_end_tag ) => {
        $self.current_token = Some(ShallowToken::EndTag {
            name: SliceRange::default(),
        });
    };


    // Tag-related
    //--------------------------------------------------------------------
    ( | $self:tt |> finish_tag_name ) => {
        match $self.current_token {
            Some(ShallowToken::StartTag { ref mut name, .. }) |
            Some(ShallowToken::EndTag { ref mut name, .. }) => {
                action_helper!(@set_token_part_range |$self|> name);
            }
            _ => unreachable!("Current token should always be a start or an end tag at this point")
        }
    };

    ( | $self:tt |> mark_as_self_closing ) => {
        if let Some(ShallowToken::StartTag { ref mut self_closing, .. }) = $self.current_token {
            *self_closing = true;
        }
    };


    // Attributes
    //--------------------------------------------------------------------
    ( | $self:tt |> start_attr ) => {
        // NOTE: create attribute only if we are parsing a start tag
        if let Some(ShallowToken::StartTag {..}) = $self.current_token {
            $self.current_attr = Some(ShallowAttribute::default());
            action!(|$self|> start_token_part);
        }
    };

    ( | $self:tt |> finish_attr_name ) => {
        action_helper!(@finish_attr_part |$self|> name);
    };

    ( | $self:tt |> finish_attr_value ) => {
        action_helper!(@finish_attr_part |$self|> value);
    };

    ( | $self:tt |> finish_attr ) => {
        match $self.current_attr.take() {
            Some(attr) => {
                $self.attr_buffer.borrow_mut().push(attr);
            }
            // NOTE: end tag case
            None => ()
        }
    };


    // Quotes
    //--------------------------------------------------------------------
    ( | $self:tt |> set_closing_quote_to_double ) => {
        $self.closing_quote = b'"';
    };

    ( | $self:tt |> set_closing_quote_to_single ) => {
        $self.closing_quote = b'\'';
    };

}
