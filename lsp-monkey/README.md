### lsp-monkey

lsp-monkey is an implementation of the LSP protocol both in general 
and for the monkey programming language.  In this project, I define 
all the types and functions needed to implement an LPS server, and 
I do so for monkey.

### TODO 

- [x] Go through all types, using #[serder(rename="")] where needed
- [x] Go through all types, using #[serde(default, skip_serializing_if)] as needed
- [ ] Implement InitializeParams
    - [ ] Implement ClientCapabilities
    - [ ] Implement TraceValues
    - [ ] Implement WorkspaceFolders

On pause until HTTP1.1 is implemented.  I have too many side projects and this 
one is way too big.

