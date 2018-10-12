pub mod tokenizer;

// TODO
// -- Functionality
// 1. Move tag name to tokenizer
// 2. Streaming
// 3. Eager tokenizer
// 4. Tokenizer driver
// 5. Adjustable limits
// 6. Get rid of token view as we don't need to store buffer anymore
//
// -- Performance
// 1. Implement benchmark
// 2. LTO
// 3. In-state loops
// 4. Don't emit character immidiately, extend existing
// 5. State embedding
