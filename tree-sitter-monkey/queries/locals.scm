; Scopes
(source_file) @local.scope
(function_expression) @local.scope
(macro_expression) @local.scope
(block) @local.scope

; Definitions
((let_statement name: (identifier) @local.definition) 
  (#has-ancestor? @local.definition block function_expression macro_expression))
(function_expression (identifier) @local.definition)
(macro_expression (identifier) @local.definition)

; References
(identifier) @local.reference
